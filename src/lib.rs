//! Autoversion - Universal semantic versioning automation
//! 
//! This library provides technology-agnostic semantic versioning automation
//! for projects regardless of their tech stack (npm, Cargo, Maven, Python, etc.)

pub mod cli;
pub mod core;
pub mod git;
pub mod updaters;
pub mod utils;

pub use core::semver::VersionBumper;
pub use core::detector::TechnologyDetector;
pub use updaters::factory::UpdaterFactory;