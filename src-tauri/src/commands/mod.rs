//! The Tauri commands the frontend invokes, grouped by what they read.
//!
//! Every command starts from a `repoId` the frontend supplies, so `paths` resolves that
//! to a folder on disk -- and refuses anything outside it (ADR 0005) -- before the other
//! modules touch the Repo.

pub mod branch;
pub mod file;
pub mod paths;
pub mod repo;
pub mod tree;
