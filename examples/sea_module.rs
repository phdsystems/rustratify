//! Example: Building a Complete SEA Module with Rustratify
//!
//! This example demonstrates how to build a file processor module
//! following the 5-layer SEA pattern using rustratify primitives.
//!
//! In a real project, each layer would be in its own crate:
//! - file-processor-common  (L1)
//! - file-processor-spi     (L2)
//! - file-processor-api     (L3)
//! - file-processor-core    (L4)
//! - file-processor         (L5 - facade)

use rustratify::prelude::*;
use std::any::Any;
use std::path::Path;

// =============================================================================
// L1: COMMON LAYER - Foundation types, errors, DTOs
// =============================================================================

/// Domain-specific error type (extends RustratifyError pattern)
#[derive(Debug, Clone)]
pub enum ProcessorError {
    /// File not found
    NotFound(String),
    /// Parse error
    ParseError(String),
    /// Unsupported file type
    Unsupported(String),
}

impl std::fmt::Display for ProcessorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound(path) => write!(f, "File not found: {}", path),
            Self::ParseError(msg) => write!(f, "Parse error: {}", msg),
            Self::Unsupported(ext) => write!(f, "Unsupported file type: {}", ext),
        }
    }
}

impl std::error::Error for ProcessorError {}

/// Result type for processor operations
pub type ProcessorResult<T> = Result<T, ProcessorError>;

/// Configuration for file processing (implements rustratify::Config)
#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    pub name: String,
    pub timeout_ms: u64,
    pub verbose: bool,
    pub include_hidden: bool,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            timeout_ms: 30000,
            verbose: false,
            include_hidden: false,
        }
    }
}

impl Config for ProcessorConfig {
    fn name(&self) -> &str {
        &self.name
    }

    fn timeout(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_millis(self.timeout_ms))
    }

    fn is_verbose(&self) -> bool {
        self.verbose
    }
}

/// Processed file output (DTO)
#[derive(Debug, Clone)]
pub struct ProcessedFile {
    pub path: String,
    pub language: String,
    pub lines: usize,
    pub tokens: usize,
}

/// Processing event for streaming
#[derive(Debug, Clone)]
pub enum ProcessorEvent {
    Started { path: String },
    Progress { path: String, percent: u8 },
    Completed { result: ProcessedFile },
    Error { path: String, message: String },
}

// =============================================================================
// L2: SPI LAYER - Provider traits (extends rustratify::Provider)
// =============================================================================

/// File processor provider trait
///
/// Extends rustratify::Provider with domain-specific processing methods.
#[async_trait]
pub trait FileProcessorProvider: Provider {
    /// Process a single file
    async fn process_file(&self, path: &Path) -> ProcessorResult<ProcessedFile>;

    /// Get supported file patterns (e.g., "*.rs", "*.py")
    fn patterns(&self) -> &[&str] {
        &[]
    }
}

// =============================================================================
// L3: API LAYER - Consumer contracts
// =============================================================================

/// Event stream for file processing
pub type ProcessorEventStream = EventStream<ProcessorEvent>;

/// Main consumer API for file processing
#[async_trait]
pub trait FileProcessor: Send + Sync {
    /// Process files and return a stream of events
    async fn process(
        &self,
        paths: Vec<String>,
        config: ProcessorConfig,
    ) -> ProcessorResult<(u32, ProcessorEventStream)>;

    /// Cancel a running process
    async fn cancel(&self, run_id: u32) -> ProcessorResult<()>;
}

// =============================================================================
// L4: CORE LAYER - Implementation
// =============================================================================

/// Rust file processor provider
#[derive(Debug)]
pub struct RustProcessor;

impl Provider for RustProcessor {
    fn name(&self) -> &str {
        "rust"
    }

    fn extensions(&self) -> &[&str] {
        &[".rs"]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl FileProcessorProvider for RustProcessor {
    async fn process_file(&self, path: &Path) -> ProcessorResult<ProcessedFile> {
        // Simulated processing
        let content = std::fs::read_to_string(path)
            .map_err(|_| ProcessorError::NotFound(path.display().to_string()))?;

        Ok(ProcessedFile {
            path: path.display().to_string(),
            language: "rust".to_string(),
            lines: content.lines().count(),
            tokens: content.split_whitespace().count(),
        })
    }

    fn patterns(&self) -> &[&str] {
        &["*.rs", "**/*.rs"]
    }
}

/// Python file processor provider
#[derive(Debug)]
pub struct PythonProcessor;

impl Provider for PythonProcessor {
    fn name(&self) -> &str {
        "python"
    }

    fn extensions(&self) -> &[&str] {
        &[".py", ".pyw"]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[async_trait]
impl FileProcessorProvider for PythonProcessor {
    async fn process_file(&self, path: &Path) -> ProcessorResult<ProcessedFile> {
        let content = std::fs::read_to_string(path)
            .map_err(|_| ProcessorError::NotFound(path.display().to_string()))?;

        Ok(ProcessedFile {
            path: path.display().to_string(),
            language: "python".to_string(),
            lines: content.lines().count(),
            tokens: content.split_whitespace().count(),
        })
    }
}

/// Provider registry type alias
pub type ProcessorRegistry = Registry<dyn FileProcessorProvider>;

/// Default file processor implementation
pub struct DefaultFileProcessor {
    registry: ProcessorRegistry,
}

impl DefaultFileProcessor {
    pub fn new(registry: ProcessorRegistry) -> Self {
        Self { registry }
    }
}

#[async_trait]
impl FileProcessor for DefaultFileProcessor {
    async fn process(
        &self,
        paths: Vec<String>,
        config: ProcessorConfig,
    ) -> ProcessorResult<(u32, ProcessorEventStream)> {
        let run_id = 1; // In real impl, generate unique IDs
        let (sender, stream) = create_stream::<ProcessorEvent>();

        // Clone what we need for the spawned task
        let verbose = config.is_verbose();

        // Spawn processing task
        for path_str in paths {
            let path = Path::new(&path_str);

            // Find provider for this file
            if let Some(provider) = self.registry.find(&path_str) {
                let _ = sender
                    .send(ProcessorEvent::Started {
                        path: path_str.clone(),
                    })
                    .await;

                match provider.process_file(path).await {
                    Ok(result) => {
                        if verbose {
                            println!("Processed: {} ({} lines)", result.path, result.lines);
                        }
                        let _ = sender.send(ProcessorEvent::Completed { result }).await;
                    }
                    Err(e) => {
                        let _ = sender
                            .send(ProcessorEvent::Error {
                                path: path_str,
                                message: e.to_string(),
                            })
                            .await;
                    }
                }
            } else {
                let _ = sender
                    .send(ProcessorEvent::Error {
                        path: path_str.clone(),
                        message: format!("No processor found for: {}", path_str),
                    })
                    .await;
            }
        }

        Ok((run_id, stream))
    }

    async fn cancel(&self, _run_id: u32) -> ProcessorResult<()> {
        // In real impl, cancel the running task
        Ok(())
    }
}

// =============================================================================
// L5: FACADE LAYER - Public API (re-exports + factory functions)
// =============================================================================

/// Create a file processor with default providers
pub fn create_processor() -> impl FileProcessor {
    let mut registry: ProcessorRegistry = Registry::new();
    registry.register(Box::new(RustProcessor));
    registry.register(Box::new(PythonProcessor));
    DefaultFileProcessor::new(registry)
}

/// Create a file processor with custom registry
pub fn create_processor_with_registry(registry: ProcessorRegistry) -> impl FileProcessor {
    DefaultFileProcessor::new(registry)
}

// =============================================================================
// MAIN - Usage demonstration
// =============================================================================

#[tokio::main]
async fn main() {
    use futures::StreamExt;

    println!("=== Rustratify SEA Module Example ===\n");

    // Create processor using facade factory function
    let processor = create_processor();

    // Configure
    let config = ProcessorConfig {
        name: "example".to_string(),
        verbose: true,
        ..Default::default()
    };

    // Process files (using this example file itself)
    let files = vec!["examples/sea_module.rs".to_string()];

    match processor.process(files, config).await {
        Ok((run_id, mut stream)) => {
            println!("Started processing (run_id: {})\n", run_id);

            while let Some(event) = stream.next().await {
                match event {
                    ProcessorEvent::Started { path } => {
                        println!("📄 Processing: {}", path);
                    }
                    ProcessorEvent::Progress { path, percent } => {
                        println!("   {} - {}%", path, percent);
                    }
                    ProcessorEvent::Completed { result } => {
                        println!("✅ Completed: {}", result.path);
                        println!("   Language: {}", result.language);
                        println!("   Lines: {}", result.lines);
                        println!("   Tokens: {}", result.tokens);
                    }
                    ProcessorEvent::Error { path, message } => {
                        println!("❌ Error processing {}: {}", path, message);
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to start processing: {}", e);
        }
    }

    println!("\n=== Layer Summary ===");
    println!("L1 Common:  ProcessorError, ProcessorConfig, ProcessedFile, ProcessorEvent");
    println!("L2 SPI:     FileProcessorProvider (extends Provider)");
    println!("L3 API:     FileProcessor trait, ProcessorEventStream");
    println!("L4 Core:    RustProcessor, PythonProcessor, DefaultFileProcessor, ProcessorRegistry");
    println!("L5 Facade:  create_processor(), create_processor_with_registry()");
}
