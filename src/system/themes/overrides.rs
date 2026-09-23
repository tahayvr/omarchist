//! Optional per-app files in a theme folder.
//!
//! `omarchy-theme-set-templates` renders every `default/themed/*.tpl` into the
//! staged theme unless the theme already ships a file of that name, and
//! replaces one `[section]` of the generated `shell.toml` for each
//! `shell.<section>.toml` it finds. Every file here is therefore optional: when
//! it is absent Omarchy generates it from `colors.toml`.

pub mod btop;
pub mod chromium;
pub mod color_map;
pub mod entries;
pub mod files;
pub mod palette;
pub mod registry;
pub mod shell_section;
pub mod template;
pub mod validate;

pub use files::{OverrideStatus, generated, palette, read, remove, status, write};
pub use registry::{
    Category, EditorKind, Format, OVERRIDES, OverrideSpec, Seed, find, in_category,
};
pub use template::{Palette, render};
pub use validate::validate;
