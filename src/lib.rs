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
//! use scrollcast::renderer::DocumentRenderer;
//! use std::path::Path;
//!
//! fn main() -> anyhow::Result<()> {
//!     let input_path = Path::new("./my-repo");
//!     let output_path = Path::new("./output.pdf");
//!
//!     // Discover files, honouring .gitignore
//!     let processor = FileProcessor::new();
//!     let files = processor.process_directory(input_path)?;
//!
//!     // Generate the combined markdown document
//!     let generator = MarkdownGenerator::new(true, true);
//!     let markdown = generator.generate_markdown(&files, "my-repo")?;
//!
//!     // Render to the chosen format
//!     let metadata = DocumentMetadata {
//!         title: "My Repository".to_string(),
//!         ..Default::default()
//!     };
//!     let renderer = create_renderer(&OutputFormat::Pdf)?;
//!     renderer.save_to_file(&markdown, &metadata, output_path)?;
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

pub use config::Config;
pub use file_processor::FileProcessor;
pub use markdown_generator::{FileInfo, MarkdownGenerator};
pub use renderer::{create_renderer, DocumentMetadata, OutputFormat};
pub use theme::Theme;
