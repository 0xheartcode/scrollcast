use anyhow::Result;
use pulldown_cmark::{Event, html, Tag, TagEnd, CowStr};
use crate::renderer::{DocumentRenderer, DocumentMetadata};
use crate::syntax::highlighter::SyntaxHighlighter;

pub struct HtmlRenderer;

impl HtmlRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl DocumentRenderer for HtmlRenderer {
    fn render(&self, events: Vec<Event>, metadata: &DocumentMetadata) -> Result<Vec<u8>> {
        // Initialize syntax highlighter
        let highlighter = SyntaxHighlighter::new()?;
        
        // Process events to add syntax highlighting
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
        let mut body_html = String::new();
        html::push_html(&mut body_html, processed_events.into_iter());
        
        // Replace \newpage with CSS page break (handle different markdown outputs)
        body_html = body_html.replace("<p>\\newpage</p>", r#"<div style="page-break-before: always;"></div>"#);
        body_html = body_html.replace("\\newpage", r#"<div style="page-break-before: always;"></div>"#);
        
        // Create complete HTML document
        let html_document = format!(
            r#"<!DOCTYPE html>
<html lang="{}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 900px;
            margin: 0 auto;
            padding: 2rem;
            background-color: #fff;
        }}
        
        h1, h2, h3, h4, h5, h6 {{
            margin-top: 2rem;
            margin-bottom: 1rem;
            font-weight: 600;
        }}
        
        h1 {{ font-size: 2.5rem; }}
        h2 {{ font-size: 2rem; }}
        h3 {{ font-size: 1.5rem; }}
        
        pre {{
            background-color: #f6f8fa;
            padding: 1rem;
            border-radius: 6px;
            overflow-x: auto;
            line-height: 1.45;
        }}
        
        code {{
            font-family: 'SF Mono', Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
            font-size: 0.875em;
            background-color: rgba(27, 31, 35, 0.05);
            padding: 0.2em 0.4em;
            border-radius: 3px;
        }}
        
        pre code {{
            background-color: transparent;
            padding: 0;
            font-size: 0.875rem;
        }}
        
        blockquote {{
            border-left: 4px solid #dfe2e5;
            margin: 0;
            padding-left: 1rem;
            color: #6a737d;
        }}
        
        table {{
            border-collapse: collapse;
            width: 100%;
            margin: 1rem 0;
        }}
        
        table th,
        table td {{
            border: 1px solid #dfe2e5;
            padding: 0.5rem 1rem;
        }}
        
        table th {{
            background-color: #f6f8fa;
            font-weight: 600;
        }}
        
        a {{
            color: #0366d6;
            text-decoration: none;
        }}
        
        a:hover {{
            text-decoration: underline;
        }}
        
        hr {{
            border: none;
            border-top: 1px solid #e1e4e8;
            margin: 2rem 0;
        }}
        
        .metadata {{
            color: #586069;
            font-size: 0.875rem;
            margin-bottom: 2rem;
            padding-bottom: 1rem;
            border-bottom: 1px solid #e1e4e8;
        }}
        
        /* Syntect syntax highlighting styles */
        .source {{ background-color: transparent; }}
        [class*="comment"] {{ color: #6a737d; font-style: italic; }}
        [class*="keyword"] {{ color: #d73a49; font-weight: bold; }}
        [class*="storage"] {{ color: #d73a49; font-weight: bold; }}
        [class*="string"] {{ color: #032f62; }}
        [class*="constant"] {{ color: #005cc5; }}
        [class*="entity"] {{ color: #6f42c1; }}
        [class*="support"] {{ color: #005cc5; }}
        [class*="variable"] {{ color: #e36209; }}
        [class*="punctuation"] {{ color: #24292e; }}
        [class*="meta"] {{ color: #24292e; }}
        
        @media (prefers-color-scheme: dark) {{
            body {{
                background-color: #0d1117;
                color: #c9d1d9;
            }}
            
            pre {{
                background-color: #161b22;
            }}
            
            code {{
                background-color: rgba(110, 118, 129, 0.2);
            }}
            
            blockquote {{
                border-left-color: #30363d;
                color: #8b949e;
            }}
            
            table th,
            table td {{
                border-color: #30363d;
            }}
            
            table th {{
                background-color: #161b22;
            }}
            
            a {{
                color: #58a6ff;
            }}
            
            hr {{
                border-top-color: #30363d;
            }}
            
            .metadata {{
                color: #8b949e;
                border-bottom-color: #30363d;
            }}
            
            /* Dark mode syntax highlighting */
            [class*="comment"] {{ color: #8b949e; }}
            [class*="keyword"] {{ color: #ff7b72; }}
            [class*="storage"] {{ color: #ff7b72; }}
            [class*="string"] {{ color: #a5d6ff; }}
            [class*="constant"] {{ color: #79c0ff; }}
            [class*="entity"] {{ color: #d2a8ff; }}
            [class*="support"] {{ color: #79c0ff; }}
            [class*="variable"] {{ color: #ffa657; }}
            [class*="punctuation"] {{ color: #c9d1d9; }}
            [class*="meta"] {{ color: #c9d1d9; }}
        }}
    </style>
</head>
<body>
    <div class="metadata">
        <h1>{}</h1>"#,
            metadata.language,
            metadata.title,
            metadata.title
        );
        
        let mut final_html = html_document;
        
        if let Some(author) = &metadata.author {
            final_html.push_str(&format!("        <p>Author: {}</p>\n", author));
        }
        
        if let Some(date) = &metadata.date {
            final_html.push_str(&format!("        <p>Generated: {}</p>\n", date));
        }
        
        final_html.push_str("    </div>\n");
        final_html.push_str(&body_html);
        final_html.push_str("</body>\n</html>");
        
        Ok(final_html.into_bytes())
    }
}