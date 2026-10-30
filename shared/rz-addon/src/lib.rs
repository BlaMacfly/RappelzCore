//! Lua 5.1 addons with per-addon states, bounded execution and a renderer-independent UI.
//! This is an explicitly documented subset, not a WoW API compatibility layer.
mod manifest;
mod runtime;
mod saved;
mod ui;
mod xml;

pub use manifest::{Manifest, INTERFACE_VERSION};
pub use runtime::{AddonInfo, AddonRuntime, Event, EventArg, GameState, UnitState};
pub use ui::{DrawWidget, Rect, Widget, WidgetKind};
