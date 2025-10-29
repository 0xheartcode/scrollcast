use anyhow::Result;
use epub_builder::{EpubBuilder, EpubContent, ZipLibrary};
use pulldown_cmark::{Event, html, Tag, TagEnd, CowStr};
use crate::renderer::{DocumentRenderer, DocumentMetadata};
use crate::syntax::highlighter::SyntaxHighlighter;
use regex::Regex;

pub struct EpubRenderer;

impl EpubRenderer {
    pub fn new() -> Self {
        Self
    }
    
    fn convert_syntect_to_inline_css(&self, html: &str) -> String {
        let mut result = html.to_string();
        
        // Define color mappings for different syntect classes
        let class_mappings = vec![
            (r#"class="[^"]*comment[^"]*""#, r#"style="color: #6a737d; font-style: italic;""#),
            (r#"class="[^"]*keyword[^"]*""#, r#"style="color: #d73a49; font-weight: bold;""#),
            (r#"class="[^"]*storage[^"]*""#, r#"style="color: #d73a49; font-weight: bold;""#),
            (r#"class="[^"]*string[^"]*""#, r#"style="color: #032f62;""#),
            (r#"class="[^"]*constant[^"]*""#, r#"style="color: #005cc5;""#),
            (r#"class="[^"]*entity[^"]*""#, r#"style="color: #6f42c1;""#),
            (r#"class="[^"]*support[^"]*""#, r#"style="color: #005cc5;""#),
            (r#"class="[^"]*variable[^"]*""#, r#"style="color: #e36209;""#),
        ];
        
        // Apply each mapping
        for (pattern, replacement) in class_mappings {
            if let Ok(re) = Regex::new(pattern) {
                result = re.replace_all(&result, replacement).to_string();
            }
        }
        
        // Remove any remaining complex class attributes
        if let Ok(re) = Regex::new(r#"class="[^"]*""#) {
            result = re.replace_all(&result, "").to_string();
        }
        
        result
    }
}

impl DocumentRenderer for EpubRenderer {
    fn render(&self, events: Vec<Event>, metadata: &DocumentMetadata) -> Result<Vec<u8>> {
        // Initialize syntax highlighter
        let highlighter = SyntaxHighlighter::new()?;
        
        // Process events to add syntax highlighting (same as HTML renderer)
        let mut processed_events = Vec::new();
        let mut i = 0;
        
        while i < events.len() {
            match &events[i] {
                Event::Start(Tag::CodeBlock(kind)) => {
                    // Extract language from code block
                    let language = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                            if lang.is_empty() { None } else { Some(lang.as_ref()) }
                        }
                        _ => None,
                    };
                    
                    // Find the corresponding text and end events
                    i += 1;
                    let mut code_content = String::new();
                    while i < events.len() {
                        match &events[i] {
                            Event::Text(text) => {
                                code_content.push_str(text);
                            }
                            Event::End(TagEnd::CodeBlock) => {
                                break;
                            }
                            _ => {}
                        }
                        i += 1;
                    }
                    
                    // Generate highlighted HTML
                    if language.is_some() {
                        let highlighted_html = highlighter.highlight_to_html(&code_content, language);
                        let wrapped_html = format!("<pre>{}</pre>", highlighted_html);
                        processed_events.push(Event::Html(CowStr::Boxed(wrapped_html.into_boxed_str())));
                    } else {
                        // No language specified, use regular code block
                        processed_events.push(Event::Start(Tag::CodeBlock(kind.clone())));
                        processed_events.push(Event::Text(CowStr::Boxed(code_content.into_boxed_str())));
                        processed_events.push(Event::End(TagEnd::CodeBlock));
                    }
                }
                _ => {
                    processed_events.push(events[i].clone());
                }
            }
            i += 1;
        }
        
        // Convert processed events to HTML
        let mut html_output = String::new();
        html::push_html(&mut html_output, processed_events.into_iter());
        
        // Convert complex syntect spans to inline CSS for EPUB
        html_output = self.convert_syntect_to_inline_css(&html_output);
        
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