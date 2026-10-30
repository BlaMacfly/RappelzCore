use anyhow::{ensure, Context, Result};
use mlua::{ChunkMode, Lua, Table};

use crate::ui::{self, Widgets};

/// Strict subset: unsupported tags/attributes fail rather than silently changing the UI.
pub(crate) fn load(lua: &Lua, widgets: Widgets, source: &str, name: &str) -> Result<()> {
    ensure!(
        !source.contains("<!DOCTYPE") && !source.contains("<!ENTITY"),
        "XML DTD/entities are unsupported"
    );
    let document = roxmltree::Document::parse(source)?;
    let root = document.root_element();
    ensure!(root.tag_name().name() == "Ui", "XML root must be Ui");
    for child in root.children().filter(|n| n.is_element()) {
        widget(lua, widgets.clone(), child, None, name, 0)?;
    }
    Ok(())
}

fn widget(
    lua: &Lua,
    widgets: Widgets,
    node: roxmltree::Node<'_, '_>,
    parent: Option<Table>,
    name: &str,
    depth: usize,
) -> Result<()> {
    ensure!(depth < 32, "XML nesting limit exceeded");
    let kind = node.tag_name().name();
    ensure!(
        matches!(
            kind,
            "Frame" | "Button" | "FontString" | "Texture" | "StatusBar" | "GameTooltip"
        ),
        "unsupported XML widget: {kind}"
    );
    for a in node.attributes() {
        ensure!(
            matches!(a.name(), "name" | "parent" | "hidden" | "text"),
            "unsupported XML attribute: {}",
            a.name()
        );
    }
    let parent = if let Some(p) = node.attribute("parent") {
        Some(lua.globals().raw_get::<Table>(p)?)
    } else {
        parent.or_else(|| lua.globals().raw_get("UIParent").ok())
    };
    let frame = ui::create(
        lua,
        widgets.clone(),
        kind,
        node.attribute("name").map(str::to_string),
        parent,
    )?;
    if let Some(text) = node.attribute("text") {
        frame.call_method::<()>("SetText", text)?;
    }
    if node.attribute("hidden") == Some("true") {
        frame.call_method::<()>("Hide", ())?;
    }
    for child in node.children().filter(|n| n.is_element()) {
        match child.tag_name().name() {
            "Size" => {
                ensure!(
                    child.attributes().all(|a| matches!(a.name(), "x" | "y")),
                    "unsupported Size attribute"
                );
                frame.call_method::<()>(
                    "SetSize",
                    (number(child.attribute("x"))?, number(child.attribute("y"))?),
                )?;
            }
            "Anchors" => {
                for anchor in child.children().filter(|n| n.is_element()) {
                    ensure!(
                        anchor.tag_name().name() == "Anchor",
                        "unsupported Anchors child"
                    );
                    ensure!(
                        anchor.attributes().all(|a| matches!(
                            a.name(),
                            "point" | "relativeTo" | "relativePoint" | "x" | "y"
                        )),
                        "unsupported Anchor attribute"
                    );
                    ensure!(
                        !anchor.children().any(|n| n.is_element()),
                        "XML anchor children are unsupported; use x/y attributes"
                    );
                    frame.call_method::<()>(
                        "SetPoint",
                        (
                            anchor.attribute("point").unwrap_or("TOPLEFT"),
                            anchor.attribute("relativeTo").map(str::to_owned),
                            anchor.attribute("relativePoint").map(str::to_owned),
                            number(anchor.attribute("x"))?,
                            number(anchor.attribute("y"))?,
                        ),
                    )?;
                }
            }
            "Frames" => {
                for nested in child.children().filter(|n| n.is_element()) {
                    widget(
                        lua,
                        widgets.clone(),
                        nested,
                        Some(frame.clone()),
                        name,
                        depth + 1,
                    )?;
                }
            }
            "Scripts" => {
                for script in child.children().filter(|n| n.is_element()) {
                    let event = script.tag_name().name();
                    ensure!(
                        matches!(
                            event,
                            "OnLoad" | "OnEvent" | "OnClick" | "OnEnter" | "OnLeave"
                        ),
                        "unsupported XML script: {event}"
                    );
                    ensure!(
                        script.attributes().all(|a| a.name() == "function"),
                        "unsupported script attribute"
                    );
                    let function = if let Some(f) = script.attribute("function") {
                        ensure!(
                            crate::manifest::identifier(f),
                            "XML script reference must name a global function"
                        );
                        lua.globals().raw_get::<mlua::Function>(f)?
                    } else {
                        let code = script.text().unwrap_or("");
                        lua.load(format!("return function(self, ...) {code}\nend"))
                            .set_name(format!("@{name}:{event}"))
                            .set_mode(ChunkMode::Text)
                            .eval()?
                    };
                    frame.call_method::<()>("SetScript", (event, function))?;
                }
            }
            tag => anyhow::bail!("unsupported XML element: {tag}"),
        }
    }
    // OnLoad runs only after child widgets and handlers exist.
    let scripts: Table = frame.raw_get("__scripts")?;
    if let Some(onload) = scripts.raw_get::<Option<mlua::Function>>("OnLoad")? {
        onload.call::<()>(frame)?;
    }
    Ok(())
}

fn number(value: Option<&str>) -> Result<f32> {
    let value = value
        .unwrap_or("0")
        .parse::<f32>()
        .context("XML coordinate")?;
    ensure!(value.is_finite(), "non-finite XML coordinate");
    Ok(value)
}

use mlua::ObjectLike;
