//! Registry for managing providers.
//!
//! The `Registry` is a type-safe container for providers that supports
//! registration, lookup by name, and automatic selection based on capabilities.

use std::collections::HashMap;
use std::path::Path;

use crate::error::{RegistryError, RegistryResult};
use crate::provider::Provider;

/// A registry for managing providers.
///
/// The registry stores providers and provides methods for:
/// - Registration by name
/// - Lookup by name
/// - Automatic selection based on key/path matching
/// - Listing all registered providers
///
/// # Example
///
/// ```rust
/// use rustratify::{Registry, Provider};
/// use std::any::Any;
///
/// #[derive(Debug)]
/// struct MyProvider;
///
/// impl Provider for MyProvider {
///     fn name(&self) -> &str { "my-provider" }
///     fn as_any(&self) -> &dyn Any { self }
/// }
///
/// let mut registry: Registry<dyn Provider> = Registry::new();
/// registry.register(Box::new(MyProvider));
///
/// assert!(registry.get("my-provider").is_some());
/// ```
#[derive(Debug)]
pub struct Registry<P: ?Sized> {
    providers: HashMap<String, Box<P>>,
    ordered: Vec<String>,
}

impl<P: Provider + ?Sized> Registry<P> {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            ordered: Vec::new(),
        }
    }

    /// Register a provider.
    ///
    /// The provider is registered under its name. If a provider with the same
    /// name already exists, it will be replaced.
    pub fn register(&mut self, provider: Box<P>) {
        let name = provider.name().to_string();
        if !self.providers.contains_key(&name) {
            self.ordered.push(name.clone());
        }
        self.providers.insert(name, provider);
    }

    /// Register a provider, returning an error if already registered.
    pub fn register_unique(&mut self, provider: Box<P>) -> RegistryResult<()> {
        let name = provider.name().to_string();
        if self.providers.contains_key(&name) {
            return Err(RegistryError::AlreadyRegistered(name));
        }
        self.ordered.push(name.clone());
        self.providers.insert(name, provider);
        Ok(())
    }

    /// Get a provider by name.
    pub fn get(&self, name: &str) -> Option<&P> {
        self.providers.get(name).map(|p| p.as_ref())
    }

    /// Get a mutable provider by name.
    pub fn get_mut(&mut self, name: &str) -> Option<&mut P> {
        self.providers.get_mut(name).map(|p| p.as_mut())
    }

    /// Find a provider that supports the given key.
    ///
    /// Returns the first provider that returns `true` for `supports(key)`.
    /// Providers are checked in registration order.
    pub fn find(&self, key: &str) -> Option<&P> {
        self.ordered
            .iter()
            .filter_map(|name| self.providers.get(name))
            .find(|p| p.supports(key))
            .map(|p| p.as_ref())
    }

    /// Find a provider that supports the given path.
    ///
    /// Returns the first provider that returns `true` for `supports_path(path)`.
    pub fn find_by_path(&self, path: &Path) -> Option<&P> {
        self.ordered
            .iter()
            .filter_map(|name| self.providers.get(name))
            .find(|p| p.supports_path(path))
            .map(|p| p.as_ref())
    }

    /// Find the best provider for the given key, considering priority.
    ///
    /// Returns the provider with the highest priority among those that support the key.
    pub fn find_best(&self, key: &str) -> Option<&P> {
        self.ordered
            .iter()
            .filter_map(|name| self.providers.get(name))
            .filter(|p| p.supports(key))
            .max_by_key(|p| p.priority())
            .map(|p| p.as_ref())
    }

    /// Find all providers that support the given key.
    pub fn find_all(&self, key: &str) -> Vec<&P> {
        self.ordered
            .iter()
            .filter_map(|name| self.providers.get(name))
            .filter(|p| p.supports(key))
            .map(|p| p.as_ref())
            .collect()
    }

    /// Check if a provider with the given name is registered.
    pub fn contains(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    /// Remove a provider by name.
    pub fn remove(&mut self, name: &str) -> Option<Box<P>> {
        self.ordered.retain(|n| n != name);
        self.providers.remove(name)
    }

    /// Get the names of all registered providers.
    pub fn names(&self) -> Vec<&str> {
        self.ordered.iter().map(|s| s.as_str()).collect()
    }

    /// Get all registered providers.
    pub fn providers(&self) -> Vec<&P> {
        self.ordered
            .iter()
            .filter_map(|name| self.providers.get(name))
            .map(|p| p.as_ref())
            .collect()
    }

    /// Get the number of registered providers.
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    /// Clear all providers from the registry.
    pub fn clear(&mut self) {
        self.providers.clear();
        self.ordered.clear();
    }

    /// Iterate over all providers.
    pub fn iter(&self) -> impl Iterator<Item = &P> {
        self.ordered
            .iter()
            .filter_map(move |name| self.providers.get(name))
            .map(|p| p.as_ref())
    }
}

impl<P: Provider + ?Sized> Default for Registry<P> {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating registries with fluent API.
pub struct RegistryBuilder<P: ?Sized> {
    registry: Registry<P>,
}

impl<P: Provider + ?Sized> RegistryBuilder<P> {
    /// Create a new registry builder.
    pub fn new() -> Self {
        Self {
            registry: Registry::new(),
        }
    }

    /// Add a provider to the registry.
    pub fn with(mut self, provider: Box<P>) -> Self {
        self.registry.register(provider);
        self
    }

    /// Build the registry.
    pub fn build(self) -> Registry<P> {
        self.registry
    }
}

impl<P: Provider + ?Sized> Default for RegistryBuilder<P> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    #[derive(Debug)]
    struct TestProvider {
        name: String,
        extensions: Vec<&'static str>,
        priority: i32,
    }

    impl TestProvider {
        fn new(name: &str, extensions: Vec<&'static str>) -> Self {
            Self {
                name: name.to_string(),
                extensions,
                priority: 0,
            }
        }

        fn with_priority(mut self, priority: i32) -> Self {
            self.priority = priority;
            self
        }
    }

    impl Provider for TestProvider {
        fn name(&self) -> &str {
            &self.name
        }

        fn extensions(&self) -> &[&str] {
            &self.extensions
        }

        fn priority(&self) -> i32 {
            self.priority
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    #[test]
    fn test_registry_register_and_get() {
        let mut registry: Registry<dyn Provider> = Registry::new();
        registry.register(Box::new(TestProvider::new("test", vec![".test"])));

        assert!(registry.get("test").is_some());
        assert!(registry.get("unknown").is_none());
    }

    #[test]
    fn test_registry_find() {
        let mut registry: Registry<dyn Provider> = Registry::new();
        registry.register(Box::new(TestProvider::new("test", vec![".test"])));
        registry.register(Box::new(TestProvider::new("spec", vec![".spec"])));

        let provider = registry.find("file.test");
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().name(), "test");

        let provider = registry.find("file.spec");
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().name(), "spec");

        assert!(registry.find("file.unknown").is_none());
    }

    #[test]
    fn test_registry_find_best() {
        let mut registry: Registry<dyn Provider> = Registry::new();
        registry.register(Box::new(
            TestProvider::new("low", vec![".test"]).with_priority(1),
        ));
        registry.register(Box::new(
            TestProvider::new("high", vec![".test"]).with_priority(10),
        ));

        let provider = registry.find_best("file.test");
        assert!(provider.is_some());
        assert_eq!(provider.unwrap().name(), "high");
    }

    #[test]
    fn test_registry_names() {
        let mut registry: Registry<dyn Provider> = Registry::new();
        registry.register(Box::new(TestProvider::new("a", vec![])));
        registry.register(Box::new(TestProvider::new("b", vec![])));

        let names = registry.names();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[test]
    fn test_registry_builder() {
        let registry: Registry<dyn Provider> = RegistryBuilder::<dyn Provider>::new()
            .with(Box::new(TestProvider::new("a", vec![])))
            .with(Box::new(TestProvider::new("b", vec![])))
            .build();

        assert_eq!(registry.len(), 2);
    }
}
