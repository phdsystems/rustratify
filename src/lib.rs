//! # Rustratify
//!
//! **Rustratify** (Rust + Stratify) is a framework for building modular Rust applications
//! using the Stratified Encapsulation Architecture (SEA) pattern.
//!
//! ## Overview
//!
//! SEA is a 5-layer architectural pattern that provides:
//! - **Consumer isolation**: External consumers only see the facade
//! - **Provider extensibility**: New implementations via SPI traits
//! - **API stability**: Internal refactoring without breaking changes
//! - **Compile-time enforcement**: Dependency rules enforced by Rust's module system
//!
//! ## Layer Structure
//!
//! ```text
//! L5: Facade  - Consumer entry point (ONLY externally visible)
//! L4: Core    - Implementation (exports nothing directly)
//! L3: API     - Consumer contracts (async streams, traits)
//! L2: SPI     - Extension points (provider interfaces)
//! L1: Common  - Foundation (DTOs, models, errors)
//! ```
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use rustratify::prelude::*;
//!
//! // Define a provider trait (L2: SPI)
//! #[async_trait]
//! pub trait MyProvider: Provider {
//!     async fn process(&self, input: &str) -> Result<String, ProviderError>;
//! }
//!
//! // Create a registry and executor (L4: Core)
//! let mut registry = Registry::new();
//! registry.register(Box::new(MyDefaultProvider::new()));
//!
//! // Use async streams for events (L3: API)
//! let (run_id, stream) = executor.run(config).await?;
//! ```
//!
//! ## Features
//!
//! - Generic `Provider` trait for extension points
//! - Type-safe `Registry` for provider management
//! - Async stream utilities for event-driven APIs
//! - Error types following SEA conventions

mod config;
mod error;
mod provider;
mod registry;
pub mod stream;

pub mod prelude;

// Re-export core types
pub use config::{Config, ConfigBuilder, DefaultConfig, FileConfig, MergeableConfig};
pub use error::{
    ProviderError, ProviderResult, RegistryError, RegistryResult, RustratifyError, RustratifyResult,
};
pub use provider::{Provider, ProviderExt};
pub use registry::{Registry, RegistryBuilder};
pub use stream::{create_stream, EventSender, EventStream, StreamBuilder};

// Re-export async-trait for convenience
pub use async_trait::async_trait;
