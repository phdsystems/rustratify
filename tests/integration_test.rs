//! Integration tests for Rustratify
//!
//! These tests demonstrate real-world usage patterns of the SEA framework.

use rustratify::prelude::*;
use std::any::Any;
use std::path::Path;

// =============================================================================
// Test Providers
// =============================================================================

/// A file processor provider for testing
#[derive(Debug, Clone)]
struct FileProcessor {
    name: String,
    extensions: Vec<&'static str>,
    priority: i32,
}

impl FileProcessor {
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

impl Provider for FileProcessor {
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

// =============================================================================
// Registry Tests
// =============================================================================

#[test]
fn test_registry_basic_operations() {
    let mut registry: Registry<dyn Provider> = Registry::new();

    // Register providers
    registry.register(Box::new(FileProcessor::new("rust", vec![".rs"])));
    registry.register(Box::new(FileProcessor::new("python", vec![".py"])));
    registry.register(Box::new(FileProcessor::new("javascript", vec![".js", ".mjs"])));

    // Test get by name
    assert!(registry.get("rust").is_some());
    assert!(registry.get("python").is_some());
    assert!(registry.get("unknown").is_none());

    // Test contains
    assert!(registry.contains("rust"));
    assert!(!registry.contains("go"));

    // Test len and is_empty
    assert_eq!(registry.len(), 3);
    assert!(!registry.is_empty());

    // Test names
    let names = registry.names();
    assert!(names.contains(&"rust"));
    assert!(names.contains(&"python"));
    assert!(names.contains(&"javascript"));
}

#[test]
fn test_registry_find_by_extension() {
    let mut registry: Registry<dyn Provider> = Registry::new();

    registry.register(Box::new(FileProcessor::new("rust", vec![".rs"])));
    registry.register(Box::new(FileProcessor::new("typescript", vec![".ts", ".tsx"])));

    // Find by file path
    let provider = registry.find("src/main.rs");
    assert!(provider.is_some());
    assert_eq!(provider.unwrap().name(), "rust");

    let provider = registry.find("components/App.tsx");
    assert!(provider.is_some());
    assert_eq!(provider.unwrap().name(), "typescript");

    // No match
    assert!(registry.find("styles.css").is_none());
}

#[test]
fn test_registry_find_best_with_priority() {
    let mut registry: Registry<dyn Provider> = Registry::new();

    // Both handle .test files, but "specific" has higher priority
    registry.register(Box::new(
        FileProcessor::new("generic", vec![".test"]).with_priority(1),
    ));
    registry.register(Box::new(
        FileProcessor::new("specific", vec![".test"]).with_priority(10),
    ));

    let provider = registry.find_best("file.test");
    assert!(provider.is_some());
    assert_eq!(provider.unwrap().name(), "specific");
}

#[test]
fn test_registry_find_all() {
    let mut registry: Registry<dyn Provider> = Registry::new();

    registry.register(Box::new(FileProcessor::new("jest", vec![".test.js"])));
    registry.register(Box::new(FileProcessor::new("mocha", vec![".test.js"])));
    registry.register(Box::new(FileProcessor::new("vitest", vec![".test.ts"])));

    let providers = registry.find_all("spec.test.js");
    assert_eq!(providers.len(), 2);

    let names: Vec<&str> = providers.iter().map(|p| p.name()).collect();
    assert!(names.contains(&"jest"));
    assert!(names.contains(&"mocha"));
}

#[test]
fn test_registry_remove() {
    let mut registry: Registry<dyn Provider> = Registry::new();

    registry.register(Box::new(FileProcessor::new("a", vec![])));
    registry.register(Box::new(FileProcessor::new("b", vec![])));

    assert_eq!(registry.len(), 2);

    let removed = registry.remove("a");
    assert!(removed.is_some());
    assert_eq!(registry.len(), 1);
    assert!(registry.get("a").is_none());
    assert!(registry.get("b").is_some());
}

#[test]
fn test_registry_clear() {
    let mut registry: Registry<dyn Provider> = Registry::new();

    registry.register(Box::new(FileProcessor::new("a", vec![])));
    registry.register(Box::new(FileProcessor::new("b", vec![])));

    registry.clear();
    assert!(registry.is_empty());
    assert_eq!(registry.len(), 0);
}

#[test]
fn test_registry_builder() {
    let registry: Registry<dyn Provider> = RegistryBuilder::<dyn Provider>::new()
        .with(Box::new(FileProcessor::new("rust", vec![".rs"])))
        .with(Box::new(FileProcessor::new("go", vec![".go"])))
        .build();

    assert_eq!(registry.len(), 2);
    assert!(registry.get("rust").is_some());
    assert!(registry.get("go").is_some());
}

#[test]
fn test_registry_iter() {
    let mut registry: Registry<dyn Provider> = Registry::new();

    registry.register(Box::new(FileProcessor::new("a", vec![])));
    registry.register(Box::new(FileProcessor::new("b", vec![])));
    registry.register(Box::new(FileProcessor::new("c", vec![])));

    let names: Vec<&str> = registry.iter().map(|p| p.name()).collect();
    assert_eq!(names, vec!["a", "b", "c"]);
}

#[test]
fn test_registry_unique_registration() {
    let mut registry: Registry<dyn Provider> = Registry::new();

    let result = registry.register_unique(Box::new(FileProcessor::new("test", vec![])));
    assert!(result.is_ok());

    let result = registry.register_unique(Box::new(FileProcessor::new("test", vec![])));
    assert!(result.is_err());
}

// =============================================================================
// Provider Tests
// =============================================================================

#[test]
fn test_provider_downcast() {
    let provider = FileProcessor::new("test", vec![".txt"]);

    // Test is<T>
    assert!(provider.is::<FileProcessor>());

    // Test downcast_ref
    let concrete = provider.downcast_ref::<FileProcessor>();
    assert!(concrete.is_some());
    assert_eq!(concrete.unwrap().name, "test");
}

#[test]
fn test_provider_supports_path() {
    let provider = FileProcessor::new("rust", vec![".rs"]);

    assert!(provider.supports_path(Path::new("src/main.rs")));
    assert!(provider.supports_path(Path::new("/absolute/path/lib.rs")));
    assert!(!provider.supports_path(Path::new("main.py")));
}

#[test]
fn test_cloneable_provider_integration() {
    let provider = FileProcessor::new("rust", vec![".rs", ".toml"]).with_priority(10);

    // Box as CloneableProvider trait object
    let boxed: Box<dyn CloneableProvider> = Box::new(provider);

    // Clone the provider
    let cloned = boxed.clone_box();

    // Verify the clone has the same properties
    assert_eq!(cloned.name(), "rust");
    assert_eq!(cloned.extensions(), &[".rs", ".toml"]);
    assert_eq!(cloned.priority(), 10);

    // Verify the clone works with provider methods
    assert!(cloned.supports("main.rs"));
    assert!(cloned.supports("Cargo.toml"));
    assert!(!cloned.supports("main.py"));

    // Verify we can downcast the clone
    assert!(cloned.is::<FileProcessor>());
    let concrete = cloned.downcast_ref::<FileProcessor>();
    assert!(concrete.is_some());
    assert_eq!(concrete.unwrap().name, "rust");
}

#[test]
fn test_registry_with_cloneable_providers() {
    let mut registry: Registry<dyn Provider> = Registry::new();

    // Register cloneable providers
    let rust_provider = FileProcessor::new("rust", vec![".rs"]);
    let python_provider = FileProcessor::new("python", vec![".py"]).with_priority(5);

    registry.register(Box::new(rust_provider.clone()));
    registry.register(Box::new(python_provider.clone()));

    // Clone a provider from the registry for modification
    let boxed: Box<dyn CloneableProvider> = Box::new(rust_provider);
    let cloned = boxed.clone_box();

    // Original registry is unchanged
    assert_eq!(registry.len(), 2);
    assert_eq!(registry.get("rust").unwrap().name(), "rust");

    // We can work with the clone independently
    assert_eq!(cloned.name(), "rust");
}

// =============================================================================
// Stream Tests
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
enum TestEvent {
    Started,
    Progress(u32),
    Completed(String),
}

#[tokio::test]
async fn test_event_stream_basic() {
    use futures::StreamExt;

    let (sender, mut stream) = create_stream::<TestEvent>();

    // Send events
    sender.send(TestEvent::Started).await.unwrap();
    sender.send(TestEvent::Progress(50)).await.unwrap();
    sender.send(TestEvent::Completed("done".to_string())).await.unwrap();
    drop(sender); // Close the stream

    // Receive events
    let mut events = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event);
    }

    assert_eq!(events.len(), 3);
    assert_eq!(events[0], TestEvent::Started);
    assert_eq!(events[1], TestEvent::Progress(50));
    assert_eq!(events[2], TestEvent::Completed("done".to_string()));
}

#[tokio::test]
async fn test_event_stream_with_buffer() {
    use futures::StreamExt;

    let (sender, stream) = StreamBuilder::<TestEvent>::new()
        .buffer_size(10)
        .build();

    // Send multiple events
    for i in 0..5 {
        sender.send(TestEvent::Progress(i * 20)).await.unwrap();
    }
    drop(sender);

    let count = stream.collect::<Vec<_>>().await.len();
    assert_eq!(count, 5);
}

#[tokio::test]
async fn test_event_sender_clone() {
    use futures::StreamExt;

    let (sender1, stream) = create_stream::<TestEvent>();
    let sender2 = sender1.clone();

    sender1.send(TestEvent::Started).await.unwrap();
    sender2.send(TestEvent::Progress(100)).await.unwrap();

    drop(sender1);
    drop(sender2);

    let events: Vec<_> = stream.collect().await;
    assert_eq!(events.len(), 2);
}

#[tokio::test]
async fn test_try_send() {
    let (sender, _stream) = create_stream::<TestEvent>();

    // try_send should succeed when buffer has space
    let result = sender.try_send(TestEvent::Started);
    assert!(result.is_ok());
}

// =============================================================================
// Error Tests
// =============================================================================

#[test]
fn test_registry_error_display() {
    let error = RegistryError::AlreadyRegistered("test".to_string());
    let msg = format!("{}", error);
    assert!(msg.contains("test"));
    assert!(msg.contains("already registered"));
}

#[test]
fn test_provider_error_display() {
    let error = ProviderError::NotSupported("test".to_string());
    let msg = format!("{}", error);
    assert!(msg.contains("test"));
}

// =============================================================================
// Real-World Scenario Tests
// =============================================================================

/// Simulates a multi-language code processor
#[test]
fn test_multi_language_processor_scenario() {
    let mut registry: Registry<dyn Provider> = Registry::new();

    // Register language processors
    registry.register(Box::new(FileProcessor::new("rust", vec![".rs"])));
    registry.register(Box::new(FileProcessor::new("python", vec![".py", ".pyw"])));
    registry.register(Box::new(FileProcessor::new("javascript", vec![".js", ".jsx"])));
    registry.register(Box::new(FileProcessor::new("typescript", vec![".ts", ".tsx"])));

    // Process files
    let files = vec![
        "src/main.rs",
        "lib/utils.py",
        "components/App.tsx",
        "scripts/build.js",
        "data/config.json", // No processor
    ];

    let mut processed = Vec::new();
    for file in files {
        if let Some(provider) = registry.find(file) {
            processed.push((file, provider.name()));
        }
    }

    assert_eq!(processed.len(), 4);
    assert!(processed.contains(&("src/main.rs", "rust")));
    assert!(processed.contains(&("lib/utils.py", "python")));
    assert!(processed.contains(&("components/App.tsx", "typescript")));
    assert!(processed.contains(&("scripts/build.js", "javascript")));
}

/// Simulates provider selection with fallback
#[test]
fn test_provider_with_fallback() {
    let mut registry: Registry<dyn Provider> = Registry::new();

    // Specialized TypeScript handler (high priority)
    registry.register(Box::new(
        FileProcessor::new("typescript-strict", vec![".ts"]).with_priority(10),
    ));

    // Generic JavaScript/TypeScript handler (lower priority)
    registry.register(Box::new(
        FileProcessor::new("ecmascript", vec![".js", ".ts"]).with_priority(1),
    ));

    // TypeScript files get the specialized handler
    let ts_handler = registry.find_best("app.ts");
    assert_eq!(ts_handler.unwrap().name(), "typescript-strict");

    // JavaScript files get the generic handler
    let js_handler = registry.find_best("app.js");
    assert_eq!(js_handler.unwrap().name(), "ecmascript");
}
