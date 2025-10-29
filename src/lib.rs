//! # Scrollcast
//! 
//! A fast Rust library for converting Git repositories to beautifully formatted documents.
//! 
//! ## Features
//! 
//! - Convert repositories to PDF, EPUB, HTML, and Markdown
//! - Syntax highlighting for 300+ programming languages
//! - Git integration with .gitignore support
//! - Pure Rust implementation with no external dependencies
//! 
//! ## Example
//! 
//! ```rust,no_run
//! use scrollcast::{FileProcessor, MarkdownGenerator, OutputFormat, create_renderer, DocumentMetadata};
//! use std::path::Path;
//! 
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let input_path = Path::new("./my-repo");
//!     let output_path = Path::new("./output.pdf");
//!     
//!     // Process files
//!     let mut processor = FileProcessor::new(input_path, true, Vec::new())?;
//!     let files = processor.discover_files().await?;
//!     
//!     // Generate markdown
//!     let generator = MarkdownGenerator::new();
//!     let markdown = generator.generate_markdown(&files, input_path, false).await?;
//!     
//!     // Create renderer and convert
//!     let metadata = DocumentMetadata {
//!         title: "My Repository".to_string(),
//!         author: "Author".to_string(),
//!         created_at: chrono::Utc::now(),
//!     };
//!     
//!     let mut renderer = create_renderer(OutputFormat::Pdf, "kate".to_string())?;
//!     renderer.render(&markdown, output_path, &metadata).await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod config;
pub mod file_processor;
pub mod markdown_generator;
pub mod renderer;
pub mod syntax;
pub mod theme;

pub use file_processor::FileProcessor;
pub use markdown_generator::{FileInfo, MarkdownGenerator};
pub use renderer::{OutputFormat, create_renderer, DocumentMetadata};
pub use config::Config;
pub use theme::Theme;