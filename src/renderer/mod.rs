use anyhow::Result;
use std::path::Path;
use pulldown_cmark::Event;

pub mod pdf;
pub mod epub;
pub mod html;

/// Metadata for document generation
#[derive(Debug, Clone)]
pub struct DocumentMetadata {
    pub title: String,
    pub author: Option<String>,
    pub date: Option<String>,
    pub language: String,
    pub include_toc: bool,
    pub syntax_theme: String,
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            title: "Document".to_string(),
            author: None,
            date: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
            language: "en".to_string(),
            include_toc: true,
            syntax_theme: "InspiredGitHub".to_string(),
        }
    }
}

/// Trait for document renderers
pub trait DocumentRenderer {
    /// Render markdown events to the target format
    fn render(&self, events: Vec<Event>, metadata: &DocumentMetadata) -> Result<Vec<u8>>;
    
    /// Render markdown string to the target format
    fn render_markdown(&self, markdown: &str, metadata: &DocumentMetadata) -> Result<Vec<u8>> {
        use pulldown_cmark::{Parser, Options};
        
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        options.insert(Options::ENABLE_FOOTNOTES);
        options.insert(Options::ENABLE_STRIKETHROUGH);
        options.insert(Options::ENABLE_TASKLISTS);
        options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
        
        let parser = Parser::new_ext(markdown, options);
        let events: Vec<Event> = parser.collect();
        
        self.render(events, metadata)
    }
    
    /// Save rendered document to file
    fn save_to_file(&self, markdown: &str, metadata: &DocumentMetadata, output_path: &Path) -> Result<()> {
        let rendered = self.render_markdown(markdown, metadata)?;
        std::fs::write(output_path, rendered)?;
        Ok(())
    }
}

/// Output format for documents
#[derive(Debug, Clone)]
pub enum OutputFormat {
    Pdf,
    Epub,
    Html,
    Markdown,
}

impl OutputFormat {
    pub fn extension(&self) -> &str {
        match self {
            OutputFormat::Pdf => "pdf",
            OutputFormat::Epub => "epub", 
            OutputFormat::Html => "html",
            OutputFormat::Markdown => "md",
        }
    }
}

/// Factory for creating renderers based on output format
pub fn create_renderer(format: &OutputFormat) -> Result<Box<dyn DocumentRenderer>> {
    match format {
        OutputFormat::Pdf => Ok(Box::new(pdf::PdfRenderer::new()?)),
        OutputFormat::Epub => Ok(Box::new(epub::EpubRenderer::new())),
        OutputFormat::Html => Ok(Box::new(html::HtmlRenderer::new())),
        OutputFormat::Markdown => {
            anyhow::bail!("Markdown output doesn't need a renderer")
        }
    }
}