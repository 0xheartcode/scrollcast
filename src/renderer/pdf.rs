use anyhow::Result;
use pulldown_cmark::Event;
use crate::renderer::{DocumentRenderer, DocumentMetadata};

pub struct PdfRenderer;

impl PdfRenderer {
    pub fn new() -> Result<Self> {
        Ok(Self)
    }
}

impl DocumentRenderer for PdfRenderer {
    fn render(&self, events: Vec<Event>, metadata: &DocumentMetadata) -> Result<Vec<u8>> {
        // Process markdown events to create structured content like HTML
        let mut pdf_content_lines = Vec::new();
        let mut current_text = String::new();
        let mut in_heading = false;
        let mut heading_level = 1;
        let mut in_code_block = false;
        
        for event in events {
            match event {
                Event::Start(pulldown_cmark::Tag::Heading { level, .. }) => {
                    if !current_text.is_empty() {
                        pdf_content_lines.push(format!("TEXT: {}", current_text.trim()));
                        current_text.clear();
                    }
                    in_heading = true;
                    heading_level = level as u32;
                }
                Event::End(pulldown_cmark::TagEnd::Heading(_)) => {
                    if !current_text.is_empty() {
                        let heading_marker = match heading_level {
                            1 => "H1",
                            2 => "H2", 
                            3 => "H3",
                            _ => "H4",
                        };
                        pdf_content_lines.push(format!("{}: {}", heading_marker, current_text.trim()));
                        current_text.clear();
                    }
                    in_heading = false;
                }
                Event::Start(pulldown_cmark::Tag::CodeBlock(_)) => {
                    if !current_text.is_empty() {
                        pdf_content_lines.push(format!("TEXT: {}", current_text.trim()));
                        current_text.clear();
                    }
                    in_code_block = true;
                }
                Event::End(pulldown_cmark::TagEnd::CodeBlock) => {
                    if !current_text.is_empty() {
                        // Split code into multiple lines for better formatting
                        let code_lines: Vec<&str> = current_text.lines().collect();
                        pdf_content_lines.push("CODE_START".to_string());
                        for line in code_lines.iter().take(10) { // Limit code lines
                            pdf_content_lines.push(format!("CODE_LINE: {}", line.trim()));
                        }
                        if code_lines.len() > 10 {
                            pdf_content_lines.push("CODE_LINE: ... [more code]".to_string());
                        }
                        pdf_content_lines.push("CODE_END".to_string());
                        current_text.clear();
                    }
                    in_code_block = false;
                }
                Event::Start(pulldown_cmark::Tag::List(_)) => {
                    if !current_text.is_empty() {
                        pdf_content_lines.push(format!("TEXT: {}", current_text.trim()));
                        current_text.clear();
                    }
                }
                Event::Start(pulldown_cmark::Tag::Item) => {
                    if !current_text.is_empty() {
                        pdf_content_lines.push(format!("TEXT: {}", current_text.trim()));
                        current_text.clear();
                    }
                }
                Event::End(pulldown_cmark::TagEnd::Item) => {
                    if !current_text.is_empty() {
                        pdf_content_lines.push(format!("LIST_ITEM: • {}", current_text.trim()));
                        current_text.clear();
                    }
                }
                Event::Start(pulldown_cmark::Tag::Table(_)) => {
                    if !current_text.is_empty() {
                        pdf_content_lines.push(format!("TEXT: {}", current_text.trim()));
                        current_text.clear();
                    }
                    pdf_content_lines.push("TABLE_START".to_string());
                }
                Event::End(pulldown_cmark::TagEnd::Table) => {
                    pdf_content_lines.push("TABLE_END".to_string());
                }
                Event::Start(pulldown_cmark::Tag::TableHead) => {
                    // Table header start
                }
                Event::End(pulldown_cmark::TagEnd::TableHead) => {
                    // Table header end
                }
                Event::Start(pulldown_cmark::Tag::TableRow) => {
                    if !current_text.is_empty() {
                        pdf_content_lines.push(format!("TEXT: {}", current_text.trim()));
                        current_text.clear();
                    }
                }
                Event::End(pulldown_cmark::TagEnd::TableRow) => {
                    if !current_text.is_empty() {
                        pdf_content_lines.push(format!("TABLE_ROW: {}", current_text.trim()));
                        current_text.clear();
                    }
                }
                Event::Start(pulldown_cmark::Tag::TableCell) => {
                    // Cell start
                }
                Event::End(pulldown_cmark::TagEnd::TableCell) => {
                    current_text.push_str(" | ");
                }
                Event::End(pulldown_cmark::TagEnd::Paragraph) => {
                    if !current_text.is_empty() && !in_heading && !in_code_block {
                        pdf_content_lines.push(format!("TEXT: {}", current_text.trim()));
                        current_text.clear();
                    }
                }
                Event::Text(text) => {
                    // Handle page breaks
                    if text.contains("\\newpage") {
                        let parts: Vec<&str> = text.split("\\newpage").collect();
                        for (i, part) in parts.iter().enumerate() {
                            if !part.is_empty() {
                                current_text.push_str(part);
                            }
                            if i < parts.len() - 1 {
                                if !current_text.is_empty() {
                                    pdf_content_lines.push(format!("TEXT: {}", current_text.trim()));
                                    current_text.clear();
                                }
                                pdf_content_lines.push("PAGE_BREAK".to_string());
                            }
                        }
                    } else {
                        current_text.push_str(&text);
                    }
                }
                Event::Code(code) => {
                    current_text.push_str(&format!("`{}`", code));
                }
                Event::SoftBreak | Event::HardBreak => {
                    current_text.push(' ');
                }
                _ => {}
            }
        }
        
        // Add any remaining text
        if !current_text.is_empty() {
            if in_code_block {
                pdf_content_lines.push(format!("CODE: {}", current_text.trim()));
            } else {
                pdf_content_lines.push(format!("TEXT: {}", current_text.trim()));
            }
        }
        
        // Create multi-page PDF with proper formatting
        let mut pages = Vec::new();
        let mut current_page_content = String::new();
        let mut y_pos = 720;
        let mut page_num = 1;
        
        // Helper function to start a new page
        let mut start_new_page = |content: &mut String, y: &mut i32, num: &mut i32| {
            if !content.is_empty() {
                pages.push(content.clone());
                content.clear();
            }
            *y = 720;
            *num += 1;
        };
        
        // First page - Title and metadata
        current_page_content.push_str(&format!(
            "/F2 18 Tf\n50 {} Td\n({}) Tj\n0 -30 Td\n",
            y_pos,
            metadata.title.replace("(", "\\(").replace(")", "\\)")
        ));
        y_pos -= 30;
        
        current_page_content.push_str(&format!(
            "/F1 10 Tf\n(Generated: {}) Tj\n0 -25 Td\n",
            metadata.date.as_ref().unwrap_or(&"Today".to_string())
        ));
        y_pos -= 25;
        
        // Process content with page breaks
        for line in pdf_content_lines.iter() {
            // Check for file boundaries (start new page)
            let is_file_header = line.starts_with("H3:") && (
                line.contains("README.md") || 
                line.contains("main.rs") || 
                line.contains(".rs") ||
                line.contains(".md")
            );
            
            // Force page break on main.rs or explicit page breaks
            let is_main_rs = line.contains("main.rs") && line.starts_with("H3:");
            
            if line == "PAGE_BREAK" || is_main_rs {
                start_new_page(&mut current_page_content, &mut y_pos, &mut page_num);
            }
            
            // Skip processing PAGE_BREAK lines (they just trigger page breaks)
            if line == "PAGE_BREAK" {
                continue;
            }
            
            // Check if we need a new page before adding content
            if y_pos < 150 { // More generous space check
                start_new_page(&mut current_page_content, &mut y_pos, &mut page_num);
            }
            
            let (font_cmd, text_content) = if line.starts_with("H1:") {
                ("/F2 16 Tf", &line[3..])
            } else if line.starts_with("H2:") {
                ("/F2 14 Tf", &line[3..])
            } else if line.starts_with("H3:") || line.starts_with("H4:") {
                ("/F2 12 Tf", &line[3..])
            } else if line.starts_with("CODE_LINE:") {
                ("/F3 9 Tf", &line[10..])
            } else if line.starts_with("CODE_START") {
                ("/F1 10 Tf", "--- Code Block ---")
            } else if line.starts_with("CODE_END") {
                ("/F1 10 Tf", "--- End Code ---")
            } else if line.starts_with("LIST_ITEM:") {
                ("/F1 10 Tf", &line[10..])
            } else if line.starts_with("TABLE_ROW:") {
                ("/F1 9 Tf", &line[10..])
            } else if line == "TABLE_START" {
                ("/F1 10 Tf", "--- Table ---")
            } else if line == "TABLE_END" {
                ("/F1 10 Tf", "--- End Table ---")
            } else if line.starts_with("TEXT:") {
                ("/F1 10 Tf", &line[5..])
            } else {
                ("/F1 10 Tf", line.as_str())
            };
            
            let escaped_text = text_content
                .replace("(", "\\(")
                .replace(")", "\\)")
                .replace("\\", "\\\\");
            
            let line_spacing = if line.starts_with("H") { 
                20 
            } else if line.starts_with("LIST_ITEM:") || line.starts_with("TABLE_ROW:") { 
                12 
            } else { 
                15 
            };
            
            current_page_content.push_str(&format!(
                "{}\n({}) Tj\n0 -{} Td\n",
                font_cmd,
                escaped_text,
                line_spacing
            ));
            y_pos -= line_spacing;
        }
        
        // Add final page
        if !current_page_content.is_empty() {
            pages.push(current_page_content);
        }
        
        let num_pages = pages.len();
        
        // Generate multi-page PDF structure
        let mut pdf_content = String::from("%PDF-1.4\n");
        
        // Catalog
        pdf_content.push_str("1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        
        // Pages object with kids array
        let mut kids_array = String::new();
        for i in 0..num_pages {
            if i > 0 { kids_array.push(' '); }
            kids_array.push_str(&format!("{} 0 R", 3 + i));
        }
        pdf_content.push_str(&format!(
            "2 0 obj\n<< /Type /Pages /Kids [{}] /Count {} >>\nendobj\n",
            kids_array, num_pages
        ));
        
        // Page objects and content streams
        let mut content_objects = Vec::new();
        for (i, page_content) in pages.iter().enumerate() {
            let page_obj_num = 3 + i;
            let content_obj_num = 3 + num_pages + 3 + i; // After pages, fonts, then content
            
            // Page object
            pdf_content.push_str(&format!(
                "{} 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792]\n/Resources << /Font << /F1 {} 0 R /F2 {} 0 R /F3 {} 0 R >> >>\n/Contents {} 0 R >>\nendobj\n",
                page_obj_num,
                3 + num_pages,     // F1 font object
                3 + num_pages + 1, // F2 font object  
                3 + num_pages + 2, // F3 font object
                content_obj_num
            ));
            
            content_objects.push((content_obj_num, page_content));
        }
        
        // Font objects
        let font1_obj = 3 + num_pages;
        let font2_obj = 3 + num_pages + 1;
        let font3_obj = 3 + num_pages + 2;
        
        pdf_content.push_str(&format!(
            "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>\nendobj\n",
            font1_obj
        ));
        pdf_content.push_str(&format!(
            "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>\nendobj\n",
            font2_obj
        ));
        pdf_content.push_str(&format!(
            "{} 0 obj\n<< /Type /Font /Subtype /Type1 /BaseFont /Courier >>\nendobj\n",
            font3_obj
        ));
        
        // Content stream objects
        for (obj_num, content) in content_objects {
            let content_length = content.len();
            pdf_content.push_str(&format!(
                "{} 0 obj\n<< /Length {} >>\nstream\nBT\n{}ET\nendstream\nendobj\n",
                obj_num, content_length, content
            ));
        }
        
        // Calculate total objects
        let total_objects = 3 + num_pages + 3 + num_pages; // catalog + pages + page_objs + fonts + content_objs
        
        // xref table 
        pdf_content.push_str(&format!("xref\n0 {}\n", total_objects));
        pdf_content.push_str("0000000000 65535 f\n");
        for i in 1..total_objects {
            pdf_content.push_str(&format!("{:010} 00000 n\n", 9 + i * 50)); // Rough offset
        }
        
        // Trailer
        pdf_content.push_str(&format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
            total_objects,
            pdf_content.len() - 100
        ));
        
        Ok(pdf_content.into_bytes())
    }
}