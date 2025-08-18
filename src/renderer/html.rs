use anyhow::Result;
use pulldown_cmark::{Event, html, Parser, Tag, TagEnd, CodeBlockKind};
use crate::renderer::{DocumentRenderer, DocumentMetadata};
use crate::syntax::highlighter::SyntaxHighlighter;

pub struct HtmlRenderer;

impl HtmlRenderer {
    pub fn new() -> Self {
        Self
    }
    
    fn process_markdown_with_syntax_highlighting(&self, markdown: &str) -> Result<String> {
        let parser = Parser::new(markdown);
        let mut events = Vec::new();
        let mut in_code_block = false;
        let mut code_lang = String::new();
        let mut code_content = String::new();
        
        // Initialize syntax highlighter
        let highlighter = SyntaxHighlighter::new()
            .map_err(|e| anyhow::anyhow!("Failed to create syntax highlighter: {}", e))?;
        
        for event in parser {
            match event {
                Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                    in_code_block = true;
                    code_lang = lang.to_string();
                    code_content.clear();
                }
                Event::End(TagEnd::CodeBlock) => {
                    if in_code_block {
                        // Apply syntax highlighting
                        let highlighted = if !code_lang.is_empty() {
                            highlighter.highlight_to_html(&code_content, Some(&code_lang))
                        } else {
                            code_content.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
                        };
                        
                        // Add syntax highlighting CSS classes and wrap in proper HTML
                        let html_block = format!(
                            r#"<div class="highlight"><pre class="code-block language-{}"><code class="language-{}">{}</code></pre></div>"#,
                            code_lang, code_lang, highlighted
                        );
                        events.push(Event::Html(html_block.into()));
                        
                        in_code_block = false;
                    }
                }
                Event::Text(text) if in_code_block => {
                    code_content.push_str(&text);
                }
                _ => {
                    if !in_code_block {
                        events.push(event);
                    }
                }
            }
        }
        
        let mut html_output = String::new();
        html::push_html(&mut html_output, events.into_iter());
        
        Ok(html_output)
    }
}

impl DocumentRenderer for HtmlRenderer {
    fn render(&self, events: Vec<Event>, metadata: &DocumentMetadata) -> Result<Vec<u8>> {
        // Convert markdown events to HTML body
        let mut body_html = String::new();
        html::push_html(&mut body_html, events.into_iter());
        
        // Replace \newpage with CSS page break (handle different markdown outputs)
        body_html = body_html.replace("<p>\\newpage</p>", r#"<div style="page-break-before: always;"></div>"#);
        body_html = body_html.replace("\\newpage", r#"<div style="page-break-before: always;"></div>"#);
    
    fn render_markdown(&self, markdown: &str, metadata: &DocumentMetadata) -> Result<Vec<u8>> {
        // Process markdown with syntax highlighting
        let mut body_html = self.process_markdown_with_syntax_highlighting(markdown)?;
        
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
        }}
        
        /* Syntax highlighting styles */
        .highlight {{
            background-color: #f8f9fa;
            border-radius: 6px;
            overflow-x: auto;
        }}
        
        .code-block {{
            background-color: transparent;
            padding: 1rem;
            margin: 0;
            font-family: 'SF Mono', Monaco, Consolas, 'Liberation Mono', 'Courier New', monospace;
            font-size: 0.875rem;
            line-height: 1.45;
        }}
        
        /* Syntect CSS classes for syntax highlighting */
        .syntect-keyword {{ color: #d73a49; font-weight: bold; }}
        .syntect-string {{ color: #032f62; }}
        .syntect-comment {{ color: #6f42c1; font-style: italic; }}
        .syntect-function {{ color: #6f42c1; }}
        .syntect-type {{ color: #005cc5; }}
        .syntect-number {{ color: #005cc5; }}
        .syntect-constant {{ color: #005cc5; }}
        .syntect-variable {{ color: #e36209; }}
        .syntect-operator {{ color: #d73a49; }}
        .syntect-preprocessor {{ color: #735c0f; }}
        
        @media (prefers-color-scheme: dark) {{
            .highlight {{
                background-color: #161b22;
            }}
            
            .syntect-keyword {{ color: #ff7b72; }}
            .syntect-string {{ color: #a5d6ff; }}
            .syntect-comment {{ color: #8b949e; }}
            .syntect-function {{ color: #d2a8ff; }}
            .syntect-type {{ color: #79c0ff; }}
            .syntect-number {{ color: #79c0ff; }}
            .syntect-constant {{ color: #79c0ff; }}
            .syntect-variable {{ color: #ffa657; }}
            .syntect-operator {{ color: #ff7b72; }}
            .syntect-preprocessor {{ color: #e3b341; }}
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