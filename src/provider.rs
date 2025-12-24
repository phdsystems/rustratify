//! Provider trait and utilities for SEA SPI layer.
//!
//! The `Provider` trait defines the contract for extension points in a Rustratify module.
//! Providers are registered in a `Registry` and selected based on their capabilities.

use std::any::Any;
use std::fmt::Debug;
use std::path::Path;

/// Base trait for all SEA providers.
///
/// Providers are extension points that implement specific functionality.
/// They are registered in a `Registry` and selected based on their capabilities.
///
/// # Example
///
/// ```rust
/// use rustratify::Provider;
/// use std::any::Any;
///
/// #[derive(Debug)]
/// struct MyProvider {
///     name: String,
/// }
///
/// impl Provider for MyProvider {
///     fn name(&self) -> &str {
///         &self.name
///     }
///
///     fn extensions(&self) -> &[&str] {
///         &[".txt", ".md"]
///     }
///
///     fn supports(&self, key: &str) -> bool {
///         key.ends_with(".txt") || key.ends_with(".md")
///     }
///
///     fn as_any(&self) -> &dyn Any {
///         self
///     }
/// }
/// ```
pub trait Provider: Send + Sync + Debug {
    /// Returns the unique name of this provider.
    ///
    /// This name is used for registration and lookup in the registry.
    fn name(&self) -> &str;

    /// Returns the file extensions this provider handles.
    ///
    /// Used for automatic provider selection based on file type.
    /// Return an empty slice if the provider doesn't use extension-based matching.
    fn extensions(&self) -> &[&str] {
        &[]
    }

    /// Check if this provider supports the given key.
    ///
    /// The key can be a file path, language name, framework name, etc.
    /// depending on the domain.
    fn supports(&self, key: &str) -> bool {
        // Default: check if key ends with any supported extension
        let extensions = self.extensions();
        if extensions.is_empty() {
            return false;
        }
        extensions.iter().any(|ext| key.ends_with(ext))
    }

    /// Check if this provider supports the given path.
    ///
    /// Override this for path-based provider selection (e.g., config file detection).
    fn supports_path(&self, path: &Path) -> bool {
        path.to_str().map(|s| self.supports(s)).unwrap_or(false)
    }

    /// Returns the priority of this provider (higher = preferred).
    ///
    /// When multiple providers match, the one with highest priority is selected.
    fn priority(&self) -> i32 {
        0
    }

    /// Downcast to concrete type for advanced usage.
    fn as_any(&self) -> &dyn Any;
}

/// Marker trait for providers that can be cloned.
///
/// This trait allows providers to be cloned behind trait objects, enabling
/// scenarios like duplicating provider configurations or creating registry snapshots.
///
/// # Example
///
/// ```rust
/// use rustratify::{Provider, CloneableProvider};
/// use std::any::Any;
///
/// #[derive(Debug, Clone)]
/// struct MyProvider {
///     name: String,
///     config: String,
/// }
///
/// impl Provider for MyProvider {
///     fn name(&self) -> &str {
///         &self.name
///     }
///
///     fn as_any(&self) -> &dyn Any {
///         self
///     }
/// }
///
/// // CloneableProvider is automatically implemented for any Provider + Clone
/// let provider = MyProvider {
///     name: "test".to_string(),
///     config: "value".to_string(),
/// };
///
/// let boxed: Box<dyn CloneableProvider> = Box::new(provider);
/// let cloned = boxed.clone_box();
/// ```
pub trait CloneableProvider: Provider {
    /// Clone the provider into a boxed trait object.
    fn clone_box(&self) -> Box<dyn CloneableProvider>;
}

impl<T> CloneableProvider for T
where
    T: Provider + Clone + 'static,
{
    fn clone_box(&self) -> Box<dyn CloneableProvider> {
        Box::new(self.clone())
    }
}

/// Extension trait for provider type checking.
pub trait ProviderExt: Provider {
    /// Check if this provider is of type T.
    fn is<T: Provider + 'static>(&self) -> bool {
        self.as_any().is::<T>()
    }

    /// Downcast to type T.
    fn downcast_ref<T: Provider + 'static>(&self) -> Option<&T> {
        self.as_any().downcast_ref::<T>()
    }
}

impl<P: Provider + ?Sized> ProviderExt for P {}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestProvider {
        name: String,
    }

    impl Provider for TestProvider {
        fn name(&self) -> &str {
            &self.name
        }

        fn extensions(&self) -> &[&str] {
            &[".test", ".spec"]
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_provider_name() {
        let provider = TestProvider {
            name: "test".to_string(),
        };
        assert_eq!(provider.name(), "test");
    }

    #[test]
    fn test_provider_supports() {
        let provider = TestProvider {
            name: "test".to_string(),
        };
        assert!(provider.supports("file.test"));
        assert!(provider.supports("file.spec"));
        assert!(!provider.supports("file.txt"));
    }

    #[test]
    fn test_provider_downcast() {
        let provider = TestProvider {
            name: "test".to_string(),
        };
        assert!(provider.is::<TestProvider>());
        assert!(provider.downcast_ref::<TestProvider>().is_some());
    }

    #[test]
    fn test_cloneable_provider() {
        let provider = TestProvider {
            name: "original".to_string(),
        };

        // Box the provider as CloneableProvider trait object
        let boxed: Box<dyn CloneableProvider> = Box::new(provider);

        // Verify we can clone it
        let cloned = boxed.clone_box();

        // Verify the clone has the same name
        assert_eq!(cloned.name(), "original");

        // Verify the clone has the same extensions
        assert_eq!(cloned.extensions(), &[".test", ".spec"]);

        // Verify the clone supports the same files
        assert!(cloned.supports("file.test"));
        assert!(!cloned.supports("file.txt"));
    }

    #[test]
    fn test_cloneable_provider_independence() {
        #[derive(Debug, Clone)]
        struct ConfigurableProvider {
            name: String,
            config_value: u32,
        }

        impl Provider for ConfigurableProvider {
            fn name(&self) -> &str {
                &self.name
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        let provider = ConfigurableProvider {
            name: "configurable".to_string(),
            config_value: 42,
        };

        let boxed: Box<dyn CloneableProvider> = Box::new(provider);
        let cloned = boxed.clone_box();

        // Verify both have the same initial value
        assert_eq!(
            boxed
                .downcast_ref::<ConfigurableProvider>()
                .unwrap()
                .config_value,
            42
        );
        assert_eq!(
            cloned
                .downcast_ref::<ConfigurableProvider>()
                .unwrap()
                .config_value,
            42
        );

        // Verify they are independent instances (different memory addresses)
        let original_ptr = boxed.as_ref() as *const dyn CloneableProvider;
        let cloned_ptr = cloned.as_ref() as *const dyn CloneableProvider;
        assert_ne!(
            original_ptr, cloned_ptr,
            "Clone should be a different instance"
        );
    }
}
