// Centralized type definitions module
// This module contains all shared type definitions used across the application

pub mod config;
pub mod errors;
pub mod hyprland;
pub mod theme;

#[cfg(test)]
mod tests;

// Re-export commonly used types for easier access
pub use config::*;
pub use errors::*;
pub use hyprland::*;
pub use theme::*;
