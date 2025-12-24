//! Prelude module for convenient imports.
//!
//! This module re-exports the most commonly used types and traits
//! from Rustratify for convenient glob imports.
//!
//! # Example
//!
//! ```rust
//! use rustratify::prelude::*;
//! ```

// Configuration
pub use crate::config::{Config, ConfigBuilder, DefaultConfig, FileConfig, MergeableConfig};

// Core traits
pub use crate::provider::{CloneableProvider, Provider, ProviderExt};

// Registry
pub use crate::registry::{Registry, RegistryBuilder};

// Streams
pub use crate::stream::{create_stream, EventSender, EventStream, EventStreamExt, StreamBuilder};

// Errors
pub use crate::error::{
    ProviderError, ProviderResult, RegistryError, RegistryResult, RustratifyError,
    RustratifyResult,
};

// Re-export async_trait for convenience
pub use async_trait::async_trait;
