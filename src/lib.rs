//! GUI-independent models and local discovery. No GTK/GIO or settings persistence.

pub mod catalog;
pub mod discovery;
pub mod image_cache;
pub mod media;
pub mod natural;
pub mod settings;
pub mod viewing;

pub const APP_NAME: &str = "Pivra";
