//! Port of `src/main/services/mcinstall/` (the `index.ts` barrel file's re-exports become plain
//! `pub use` here). Vanilla, Fabric, Quilt, Forge, and NeoForge are all implemented.

pub mod assets;
pub mod downloader;
pub mod fabric;
pub mod forge_install;
pub mod java_discovery;
pub mod java_runtime;
pub mod launch_args;
pub mod libraries;
pub mod natives;
pub mod platform;
pub mod quilt;
pub mod rules;
pub mod types;
pub mod version_resolve;
