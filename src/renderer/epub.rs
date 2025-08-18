use anyhow::Result;
use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};
use pulldown_cmark::{Event, html};
use crate::renderer::{DocumentRenderer, DocumentMetadata};

pub struct EpubRenderer;

impl EpubRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentRenderer for EpubRenderer {
    fn render(&self, events: Vec<Event>, metadata: &DocumentMetadata) -> Result<Vec<u8>> {
        // Convert markdown events to HTML
        let mut html_output = String::new();
        html::push_html(&mut html_output, events.into_iter());
        
        // Replace \newpage with EPUB page break (handle different markdown outputs)
        html_output = html_output.replace("<p>\\newpage</p>", r#"<div style="page-break-before: always;"></div>"#);
        html_output = html_output.replace("\\newpage", r#"<div style="page-break-before: always;"></div>"#);
        
        // Create EPUB
        let zip_library = ZipLibrary::new()
            .map_err(|e| anyhow::anyhow!("Failed to create zip library: {}", e))?;
        let mut builder = EpubBuilder::new(zip_library)
            .map_err(|e| anyhow::anyhow!("Failed to create EPUB builder: {}", e))?;
        
        // Set metadata
        builder.metadata("title", &metadata.title)
            .map_err(|e| anyhow::anyhow!("Failed to set title: {}", e))?;
        if let Some(author) = &metadata.author {
            builder.metadata("author", author)
                .map_err(|e| anyhow::anyhow!("Failed to set author: {}", e))?;
        }
        builder.metadata("lang", &metadata.language)
            .map_err(|e| anyhow::anyhow!("Failed to set language: {}", e))?;
        
        // Add CSS for styling with much smaller fonts and more padding for EPUB
        let css_content = r#"
            body {
                font-family: Georgia, serif;
                font-size: 0.45em;
                line-height: 1.6;
                margin: 1.5em 2em;
                padding: 1em 1.5em;
            }
            
            h1 {
                font-family: Helvetica, Arial, sans-serif;
                font-size: 0.7em;
                margin-top: 1.5em;
                margin-bottom: 0.8em;
            }
            
            h2 {
                font-family: Helvetica, Arial, sans-serif;
                font-size: 0.6em;
                margin-top: 1.2em;
                margin-bottom: 0.6em;
            }
            
            h3 {
                font-family: Helvetica, Arial, sans-serif;
                font-size: 0.55em;
                margin-top: 1em;
                margin-bottom: 0.5em;
            }
            
            pre {
                background-color: #f8f8f8;
                padding: 0.8em;
                font-size: 0.4em;
                line-height: 1.4;
                overflow-x: auto;
                border: 1px solid #e0e0e0;
                border-radius: 3px;
                margin: 0.8em 0;
            }
            
            code {
                font-family: 'Courier New', Monaco, monospace;
                font-size: 0.42em;
                background-color: #f0f0f0;
                padding: 0.2em 0.3em;
                border-radius: 2px;
            }
            
            pre code {
                background-color: transparent;
                padding: 0;
                font-size: 0.4em;
            }
            
            blockquote {
                border-left: 3px solid #ccc;
                margin-left: 0;
                padding-left: 1em;
                color: #666;
                font-style: italic;
            }
            
            table {
                width: 100%;
                border-collapse: collapse;
                font-size: 0.42em;
                margin: 0.8em 0;
            }
            
            th, td {
                border: 1px solid #ddd;
                padding: 0.5em;
                text-align: left;
            }
            
            th {
                background-color: #f5f5f5;
                font-weight: bold;
            }
            
            p {
                margin: 0.8em 0;
            }
            
            ul, ol {
                margin: 0.8em 0;
                padding-left: 2em;
            }
            
            li {
                margin: 0.3em 0;
            }
        "#;
        
        builder.stylesheet(css_content.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to add stylesheet: {}", e))?;
        
        // Add the content as a single chapter
        let chapter_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <link rel="stylesheet" type="text/css" href="stylesheet.css"/>
</head>
<body>
{}
</body>
</html>"#,
            metadata.title,
            html_output
        );
        
        builder.add_content(
            EpubContent::new("chapter1.xhtml", chapter_content.as_bytes())
                .title(&metadata.title)
        ).map_err(|e| anyhow::anyhow!("Failed to add content: {}", e))?;
        
        // Generate EPUB
        let mut buffer = Vec::new();
        builder.generate(&mut buffer)
            .map_err(|e| anyhow::anyhow!("Failed to generate EPUB: {}", e))?;
        
        Ok(buffer)
    }
}