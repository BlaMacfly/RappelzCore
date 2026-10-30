use std::cell::RefCell;
use std::rc::Rc;

use mlua::{Function, Lua, Result, Table, Value};

pub(crate) const FRAMES: &str = "rz.frames";
pub(crate) const MAX_WIDGETS: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WidgetKind {
    Frame,
    Button,
    FontString,
    Texture,
    StatusBar,
    GameTooltip,
}

#[derive(Clone, Debug)]
pub struct Widget {
    pub id: usize,
    pub name: Option<String>,
    pub kind: WidgetKind,
    pub parent: Option<usize>,
    pub visible: bool,
    pub mouse_enabled: bool,
    pub width: f32,
    pub height: f32,
    pub scale: f32,
    pub point: String,
    pub relative: usize,
    pub relative_point: String,
    pub x: f32,
    pub y: f32,
    pub text: String,
    pub color: [f32; 4],
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub events: std::collections::BTreeSet<String>,
}

pub(crate) type Widgets = Rc<RefCell<Vec<Widget>>>;

fn error(message: &str) -> mlua::Error {
    mlua::Error::RuntimeError(message.into())
}
fn finite(v: f32) -> Result<f32> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(error("UI number must be finite"))
    }
}
fn text(value: String) -> Result<String> {
    if value.len() > 4096 {
        Err(error("UI text exceeds 4096 bytes"))
    } else {
        Ok(value)
    }
}
fn anchor(value: &str) -> bool {
    matches!(
        value,
        "CENTER"
            | "TOP"
            | "BOTTOM"
            | "LEFT"
            | "RIGHT"
            | "TOPLEFT"
            | "TOPRIGHT"
            | "BOTTOMLEFT"
            | "BOTTOMRIGHT"
    )
}

pub(crate) fn create(
    lua: &Lua,
    widgets: Widgets,
    kind: &str,
    name: Option<String>,
    parent: Option<Table>,
) -> Result<Table> {
    let kind = match kind {
        "Frame" => WidgetKind::Frame,
        "Button" => WidgetKind::Button,
        "FontString" => WidgetKind::FontString,
        "Texture" => WidgetKind::Texture,
        "StatusBar" => WidgetKind::StatusBar,
        "GameTooltip" => WidgetKind::GameTooltip,
        _ => return Err(error(&format!("unsupported CreateFrame type: {kind}"))),
    };
    let frames: Table = lua.named_registry_value(FRAMES)?;
    let parent_id: Option<usize> = parent.map(|p| p.raw_get("__rz_id")).transpose()?;
    let id = widgets.borrow().len();
    if id >= MAX_WIDGETS {
        return Err(error("addon widget limit exceeded"));
    }
    if parent_id.is_some_and(|p| p >= id) {
        return Err(error("invalid widget parent"));
    }
    if let Some(name) = &name {
        if !crate::manifest::identifier(name) {
            return Err(error("widget name must be an ASCII identifier"));
        }
        if !matches!(lua.globals().raw_get::<Value>(name.as_str())?, Value::Nil) {
            return Err(error("widget name already exists"));
        }
    }
    widgets.borrow_mut().push(Widget {
        id,
        name: name.clone(),
        kind,
        parent: parent_id,
        visible: true,
        mouse_enabled: matches!(
            kind,
            WidgetKind::Frame | WidgetKind::Button | WidgetKind::GameTooltip
        ),
        width: 120.0,
        height: 24.0,
        scale: 1.0,
        point: "TOPLEFT".into(),
        relative: parent_id.unwrap_or(0),
        relative_point: "TOPLEFT".into(),
        x: 0.0,
        y: 0.0,
        text: String::new(),
        color: [1.0; 4],
        min: 0.0,
        max: 1.0,
        value: 0.0,
        events: Default::default(),
    });
    let table = lua.create_table()?;
    table.raw_set("__rz_id", id)?;
    table.raw_set("__scripts", lua.create_table()?)?;
    macro_rules! method {
        ($name:literal, $args:ty, $body:expr) => {{
            let state = widgets.clone();
            table.raw_set(
                $name,
                lua.create_function(move |lua, args: $args| {
                    let mut state = state.borrow_mut();
                    let widget = &mut state[id];
                    let apply: fn(&Lua, $args, &mut Widget) -> Result<_> = $body;
                    apply(lua, args, widget)
                })?,
            )?;
        }};
    }
    method!(
        "SetSize",
        (Table, f32, f32),
        |_: &Lua, (_, w, h), s: &mut Widget| {
            s.width = finite(w)?.clamp(0.0, 8192.0);
            s.height = finite(h)?.clamp(0.0, 8192.0);
            Ok(())
        }
    );
    method!(
        "SetWidth",
        (Table, f32),
        |_: &Lua, (_, v), s: &mut Widget| {
            s.width = finite(v)?.clamp(0.0, 8192.0);
            Ok(())
        }
    );
    method!(
        "SetHeight",
        (Table, f32),
        |_: &Lua, (_, v), s: &mut Widget| {
            s.height = finite(v)?.clamp(0.0, 8192.0);
            Ok(())
        }
    );
    method!("GetWidth", Table, |_: &Lua, _, s: &mut Widget| Ok(s.width));
    method!(
        "GetHeight",
        Table,
        |_: &Lua, _, s: &mut Widget| Ok(s.height)
    );
    method!(
        "SetScale",
        (Table, f32),
        |_: &Lua, (_, v), s: &mut Widget| {
            s.scale = finite(v)?.clamp(0.1, 4.0);
            Ok(())
        }
    );
    method!("Show", Table, |_: &Lua, _, s: &mut Widget| {
        s.visible = true;
        Ok(())
    });
    method!("Hide", Table, |_: &Lua, _, s: &mut Widget| {
        s.visible = false;
        Ok(())
    });
    method!("IsShown", Table, |_: &Lua, _, s: &mut Widget| Ok(s.visible));
    method!("GetName", Table, |_: &Lua, _, s: &mut Widget| Ok(s
        .name
        .clone()));
    method!(
        "SetText",
        (Table, String),
        |_: &Lua, (_, v), s: &mut Widget| {
            s.text = text(v)?;
            Ok(())
        }
    );
    method!("GetText", Table, |_: &Lua, _, s: &mut Widget| Ok(s
        .text
        .clone()));
    method!(
        "SetTextColor",
        (Table, f32, f32, f32, Option<f32>),
        |_: &Lua, (_, r, g, b, a): (Table, f32, f32, f32, Option<f32>), s: &mut Widget| {
            s.color = [
                finite(r)?,
                finite(g)?,
                finite(b)?,
                finite(a.unwrap_or(1.0))?,
            ]
            .map(|v| v.clamp(0.0, 1.0));
            Ok(())
        }
    );
    method!(
        "SetMinMaxValues",
        (Table, f32, f32),
        |_: &Lua, (_, min, max), s: &mut Widget| {
            s.min = finite(min)?;
            s.max = finite(max)?.max(min);
            Ok(())
        }
    );
    method!(
        "SetValue",
        (Table, f32),
        |_: &Lua, (_, value), s: &mut Widget| {
            s.value = finite(value)?;
            Ok(())
        }
    );
    method!(
        "SetColorTexture",
        (Table, f32, f32, f32, Option<f32>),
        |_: &Lua, (_, r, g, b, a): (Table, f32, f32, f32, Option<f32>), s: &mut Widget| {
            s.color = [
                finite(r)?,
                finite(g)?,
                finite(b)?,
                finite(a.unwrap_or(1.0))?,
            ]
            .map(|v| v.clamp(0.0, 1.0));
            Ok(())
        }
    );
    method!(
        "EnableMouse",
        (Table, bool),
        |_: &Lua, (_, enabled), s: &mut Widget| {
            s.mouse_enabled = enabled;
            Ok(())
        }
    );
    method!(
        "RegisterEvent",
        (Table, String),
        |_: &Lua, (_, event), s: &mut Widget| {
            if event.len() > 128 || s.events.len() >= 128 {
                return Err(error("event registration limit exceeded"));
            }
            s.events.insert(event);
            Ok(())
        }
    );
    method!(
        "UnregisterEvent",
        (Table, String),
        |_: &Lua, (_, event), s: &mut Widget| {
            s.events.remove(&event);
            Ok(())
        }
    );
    method!(
        "UnregisterAllEvents",
        Table,
        |_: &Lua, _, s: &mut Widget| {
            s.events.clear();
            Ok(())
        }
    );
    // Store callbacks only in Lua tables, avoiding Lua -> Rust -> Lua reference cycles.
    table.raw_set(
        "SetScript",
        lua.create_function(
            |_, (this, kind, function): (Table, String, Option<Function>)| {
                if !matches!(
                    kind.as_str(),
                    "OnEvent" | "OnClick" | "OnLoad" | "OnEnter" | "OnLeave"
                ) {
                    return Err(error(&format!("unsupported widget script: {kind}")));
                }
                this.raw_get::<Table>("__scripts")?.raw_set(kind, function)
            },
        )?,
    )?;
    let point_state = widgets.clone();
    table.raw_set(
        "SetPoint",
        lua.create_function(
            move |lua,
                  (this, point, relative, relative_point, x, y): (
                Table,
                String,
                Option<Value>,
                Option<String>,
                Option<f32>,
                Option<f32>,
            )| {
                if !anchor(&point) {
                    return Err(error("unsupported anchor"));
                }
                let relative_id = match relative {
                    None | Some(Value::Nil) => point_state.borrow()[id].parent.unwrap_or(0),
                    Some(Value::Table(t)) => t.raw_get("__rz_id")?,
                    Some(Value::String(s)) => lua
                        .globals()
                        .raw_get::<Table>(s.to_str()?.as_ref())?
                        .raw_get("__rz_id")?,
                    _ => return Err(error("SetPoint requires an anchor widget")),
                };
                // Anchoring only to older widgets guarantees an acyclic layout graph.
                if relative_id >= id {
                    return Err(error("anchor must refer to an earlier widget"));
                }
                let rp = relative_point.unwrap_or_else(|| point.clone());
                if !anchor(&rp) {
                    return Err(error("unsupported relative anchor"));
                }
                let _ = this;
                let mut state = point_state.borrow_mut();
                let s = &mut state[id];
                s.point = point;
                s.relative = relative_id;
                s.relative_point = rp;
                s.x = finite(x.unwrap_or(0.0))?.clamp(-16384.0, 16384.0);
                s.y = finite(y.unwrap_or(0.0))?.clamp(-16384.0, 16384.0);
                Ok(())
            },
        )?,
    )?;
    for (method, child_kind) in [
        ("CreateFontString", "FontString"),
        ("CreateTexture", "Texture"),
    ] {
        let child_state = widgets.clone();
        table.raw_set(
            method,
            lua.create_function(
                move |lua,
                      (this, name, _layer, template): (
                    Table,
                    Option<String>,
                    Option<String>,
                    Option<String>,
                )| {
                    if template.is_some() {
                        return Err(error("UI templates are not supported"));
                    }
                    create(lua, child_state.clone(), child_kind, name, Some(this))
                },
            )?,
        )?;
    }
    frames.raw_set(id + 1, table.clone())?;
    if let Some(name) = name {
        lua.globals().raw_set(name, table.clone())?;
    }
    Ok(table)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl Rect {
    pub fn contains(self, x: f32, y: f32) -> bool {
        x >= self.x && y >= self.y && x < self.x + self.width && y < self.y + self.height
    }
}

#[derive(Clone, Debug)]
pub struct DrawWidget {
    pub addon: String,
    pub widget: Widget,
    pub rect: Rect,
    pub scale: f32,
}

fn offset(anchor: &str, width: f32, height: f32) -> (f32, f32) {
    let x = if anchor.contains("LEFT") {
        0.0
    } else if anchor.contains("RIGHT") {
        width
    } else {
        width / 2.0
    };
    let y = if anchor.contains("TOP") {
        0.0
    } else if anchor.contains("BOTTOM") {
        height
    } else {
        height / 2.0
    };
    (x, y)
}

pub(crate) fn layout(addon: &str, widgets: &[Widget], width: f32, height: f32) -> Vec<DrawWidget> {
    let mut bounds = vec![Rect {
        x: 0.0,
        y: 0.0,
        width,
        height,
    }];
    let mut visible = vec![true];
    let mut scales = vec![1.0];
    let mut out = Vec::new();
    for w in widgets.iter().skip(1) {
        let parent = w.parent.unwrap_or(0);
        let relative = bounds[w.relative];
        let scale = (w.scale * scales[parent]).clamp(0.01, 16.0);
        let (width, height) = (w.width * scale, w.height * scale);
        let (rx, ry) = offset(&w.relative_point, relative.width, relative.height);
        let (px, py) = offset(&w.point, width, height);
        let rect = Rect {
            x: relative.x + rx - px + w.x * scale,
            y: relative.y + ry - py - w.y * scale,
            width,
            height,
        };
        let shown = w.visible && visible[parent];
        bounds.push(rect);
        visible.push(shown);
        scales.push(scale);
        if shown {
            out.push(DrawWidget {
                addon: addon.into(),
                widget: w.clone(),
                rect,
                scale,
            });
        }
    }
    out
}
