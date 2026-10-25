//! Models, local discovery and isolated GIO thumbnail metadata provider. No UI initialization.

pub mod catalog;
pub mod discovery;
pub mod image_cache;
pub mod media;
pub mod natural;
pub mod settings;
pub mod viewing;

pub const APP_NAME: &str = "Pivra";

pub mod image_pipeline;

// The unchanged shared image_loader boundary also builds inside this library.
extern crate self as pivra;

pub mod freedesktop_generator;
pub mod freedesktop_thumbnail;
pub mod freedesktop_writer;
pub mod thumbnail;
pub mod thumbnail_manager;
pub mod thumbnail_provider;
