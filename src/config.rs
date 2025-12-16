//! Configuration traits for SEA modules.
//!
//! This module provides base traits for configuration types used across SEA layers.

use std::path::Path;
use std::time::Duration;

/// Base trait for configuration types.
///
/// Implement this trait for your domain-specific configuration structs
/// to enable common configuration patterns.
///
/// # Example
///
/// ```rust
/// use rustratify::Config;
/// use std::time::Duration;
///
/// #[derive(Debug, Clone)]
/// struct MyConfig {
///     name: String,
///     timeout_ms: u64,
///     verbose: bool,
/// }
///
/// impl Config for MyConfig {
///     fn name(&self) -> &str {
///         &self.name
///     }
///
///     fn timeout(&self) -> Option<Duration> {
///         Some(Duration::from_millis(self.timeout_ms))
///     }
///
///     fn is_verbose(&self) -> bool {
///         self.verbose
///     }
/// }
/// ```
pub trait Config: Send + Sync {
    /// Returns the configuration name/identifier.
    fn name(&self) -> &str {
        "default"
    }

    /// Returns the timeout duration, if configured.
    fn timeout(&self) -> Option<Duration> {
        None
    }

    /// Returns whether verbose output is enabled.
    fn is_verbose(&self) -> bool {
        false
    }

    /// Returns whether debug mode is enabled.
    fn is_debug(&self) -> bool {
        false
    }

    /// Validates the configuration.
    ///
    /// Returns Ok(()) if valid, or an error message describing the issue.
    fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Trait for configurations that support file-based loading.
pub trait FileConfig: Config {
    /// Load configuration from a file path.
    fn from_file(path: &Path) -> Result<Self, String>
    where
        Self: Sized;

    /// Save configuration to a file path.
    fn to_file(&self, path: &Path) -> Result<(), String>;
}

/// Trait for configurations that can be merged.
pub trait MergeableConfig: Config {
    /// Merge another configuration into this one.
    ///
    /// Values from `other` override values in `self` where applicable.
    fn merge(&mut self, other: &Self);

    /// Create a new configuration by merging two configurations.
    fn merged(base: &Self, overlay: &Self) -> Self
    where
        Self: Clone,
    {
        let mut result = base.clone();
        result.merge(overlay);
        result
    }
}

/// Builder trait for constructing configurations.
pub trait ConfigBuilder {
    /// The configuration type this builder produces.
    type Config: Config;

    /// Build the configuration.
    fn build(self) -> Result<Self::Config, String>;
}

/// A simple default configuration implementation.
#[derive(Debug, Clone, Default)]
pub struct DefaultConfig {
    /// Configuration name
    pub name: String,
    /// Timeout in milliseconds
    pub timeout_ms: Option<u64>,
    /// Verbose output flag
    pub verbose: bool,
    /// Debug mode flag
    pub debug: bool,
}

impl DefaultConfig {
    /// Create a new default configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the configuration name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the timeout in milliseconds.
    pub fn with_timeout_ms(mut self, ms: u64) -> Self {
        self.timeout_ms = Some(ms);
        self
    }

    /// Set the timeout duration.
    pub fn with_timeout(mut self, duration: Duration) -> Self {
        self.timeout_ms = Some(duration.as_millis() as u64);
        self
    }

    /// Enable verbose output.
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Enable debug mode.
    pub fn debug(mut self) -> Self {
        self.debug = true;
        self
    }
}

impl Config for DefaultConfig {
    fn name(&self) -> &str {
        if self.name.is_empty() {
            "default"
        } else {
            &self.name
        }
    }

    fn timeout(&self) -> Option<Duration> {
        self.timeout_ms.map(Duration::from_millis)
    }

    fn is_verbose(&self) -> bool {
        self.verbose
    }

    fn is_debug(&self) -> bool {
        self.debug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DefaultConfig::new()
            .with_name("test")
            .with_timeout_ms(5000)
            .verbose();

        assert_eq!(config.name(), "test");
        assert_eq!(config.timeout(), Some(Duration::from_millis(5000)));
        assert!(config.is_verbose());
        assert!(!config.is_debug());
    }

    #[test]
    fn test_config_validation() {
        let config = DefaultConfig::new();
        assert!(config.validate().is_ok());
    }

    #[derive(Debug, Clone)]
    struct CustomConfig {
        max_workers: u32,
    }

    impl Config for CustomConfig {
        fn name(&self) -> &str {
            "custom"
        }

        fn validate(&self) -> Result<(), String> {
            if self.max_workers == 0 {
                Err("max_workers must be greater than 0".to_string())
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn test_custom_config_validation() {
        let valid = CustomConfig { max_workers: 4 };
        assert!(valid.validate().is_ok());

        let invalid = CustomConfig { max_workers: 0 };
        assert!(invalid.validate().is_err());
    }
}
