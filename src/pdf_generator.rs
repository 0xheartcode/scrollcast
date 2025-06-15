use anyhow::{Context, Result};
use printpdf::*;
use std::collections::HashMap;
use std::io::BufWriter;
use std::fs::File;

use crate::theme::Theme;

pub struct PdfGenerator {
    theme: Theme,
    include_line_numbers: bool,
    include_page_numbers: bool,
}

impl PdfGenerator {
    pub fn new(theme: Theme, include_line_numbers: bool, include_page_numbers: bool) -> Self {
        Self {
            theme,
            include_line_numbers,
            include_page_numbers,
        }
    }

    pub fn create_pdf(&self, output_path: &str, files: Vec<FileContent>) -> Result<()> {
        let (doc, page1, layer1) = PdfDocument::new(
            "Git Repository PDF",
            Mm(210.0), // A4 width
            Mm(297.0), // A4 height
            "Layer 1"
        );

        let mut current_layer = doc.get_page(page1).get_layer(layer1);
        let mut current_y = Mm(270.0); // Start near top of page
        let margin_left = Mm(20.0);
        let margin_right = Mm(190.0);
        let line_height = Mm(self.theme.font_size * self.theme.line_height);

        // Load fonts
        let font_regular = doc.add_builtin_font(BuiltinFont::Courier)?;
        let font_bold = doc.add_builtin_font(BuiltinFont::CourierBold)?;

        // Set background color (simplified approach - just use text color on white background)
        let (_bg_r, _bg_g, _bg_b) = Theme::hex_to_rgb(&self.theme.colors.background);

        for file_content in files {
            // Add file header
            current_y = self.add_file_header(&mut current_layer, &font_bold, current_y, &file_content.path)?;
            
            // Add file content
            current_y = self.add_file_content(&mut current_layer, &font_regular, current_y, &file_content)?;
            
            // Add some space between files
            current_y -= line_height * 2.0;
            
            // Check if we need a new page
            if current_y < Mm(30.0) {
                let (page, layer) = doc.add_page(Mm(210.0), Mm(297.0), "Layer 1");
                current_layer = doc.get_page(page).get_layer(layer);
                current_y = Mm(270.0);
            }
        }

        // Save the PDF
        let file = File::create(output_path)
            .context("Failed to create output PDF file")?;
        let mut writer = BufWriter::new(file);
        doc.save(&mut writer)
            .context("Failed to save PDF document")?;

        Ok(())
    }

    fn add_file_header(
        &self,
        layer: &mut PdfLayerReference,
        font: &IndirectFontRef,
        current_y: Mm,
        file_path: &str,
    ) -> Result<Mm> {
        let (header_r, header_g, header_b) = Theme::hex_to_rgb(&self.theme.colors.header);
        layer.set_fill_color(Color::Rgb(Rgb::new(header_r, header_g, header_b, None)));
        
        layer.use_text(file_path, self.theme.font_size + 2.0, Mm(20.0), current_y, font);
        
        // Add separator line (simplified - just add spacing)
        let _line_y = current_y - Mm(3.0);
        // Note: Simplified line drawing - in production version we'd use proper line drawing

        Ok(current_y - Mm(10.0))
    }

    fn add_file_content(
        &self,
        layer: &mut PdfLayerReference,
        font: &IndirectFontRef,
        mut current_y: Mm,
        file_content: &FileContent,
    ) -> Result<Mm> {
        let (text_r, text_g, text_b) = Theme::hex_to_rgb(&self.theme.colors.text);
        let line_height = Mm(self.theme.font_size * self.theme.line_height);
        let color_map = self.theme.get_color_map();

        for (line_num, line) in file_content.lines.iter().enumerate() {
            let mut x_offset = Mm(20.0);

            // Add line numbers if enabled
            if self.include_line_numbers {
                let (ln_r, ln_g, ln_b) = Theme::hex_to_rgb(&self.theme.colors.line_numbers);
                layer.set_fill_color(Color::Rgb(Rgb::new(ln_r, ln_g, ln_b, None)));
                
                let line_number_text = format!("{:4} ", line_num + 1);
                layer.use_text(&line_number_text, self.theme.font_size, x_offset, current_y, font);
                x_offset += Mm(15.0);
            }

            // Add syntax highlighted content
            layer.set_fill_color(Color::Rgb(Rgb::new(text_r, text_g, text_b, None)));
            
            if let Some(highlighted_line) = &line.highlighted_content {
                self.add_highlighted_text(layer, font, x_offset, current_y, highlighted_line, &color_map)?;
            } else {
                layer.use_text(&line.content, self.theme.font_size, x_offset, current_y, font);
            }

            current_y -= line_height;

            // Check if we need a new page
            if current_y < Mm(30.0) {
                break; // Let the caller handle page breaks
            }
        }

        Ok(current_y)
    }

    fn add_highlighted_text(
        &self,
        layer: &mut PdfLayerReference,
        font: &IndirectFontRef,
        mut x_offset: Mm,
        y: Mm,
        highlighted_content: &[HighlightedToken],
        color_map: &HashMap<String, (f32, f32, f32)>,
    ) -> Result<()> {
        for token in highlighted_content {
            let (r, g, b) = color_map.get(&token.token_type)
                .copied()
                .unwrap_or_else(|| Theme::hex_to_rgb(&self.theme.colors.text));

            layer.set_fill_color(Color::Rgb(Rgb::new(r, g, b, None)));
            layer.use_text(&token.content, self.theme.font_size, x_offset, y, font);

            // Approximate character width (this is rough, but works for monospace fonts)
            let char_width = Mm(self.theme.font_size * 0.6);
            x_offset += char_width * token.content.len() as f32;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileContent {
    pub path: String,
    pub lines: Vec<CodeLine>,
}

#[derive(Debug, Clone)]
pub struct CodeLine {
    pub content: String,
    pub highlighted_content: Option<Vec<HighlightedToken>>,
}

#[derive(Debug, Clone)]
pub struct HighlightedToken {
    pub content: String,
    pub token_type: String,
}

impl FileContent {
    pub fn new(path: String, content: String) -> Self {
        let lines = content
            .lines()
            .map(|line| CodeLine {
                content: line.to_string(),
                highlighted_content: None,
            })
            .collect();

        Self { path, lines }
    }

    pub fn with_highlighting(path: String, lines: Vec<CodeLine>) -> Self {
        Self { path, lines }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;
    use tempfile::NamedTempFile;

    #[test]
    fn test_pdf_generator_creation() {
        let theme = Theme::light();
        let generator = PdfGenerator::new(theme, true, true);
        assert!(generator.include_line_numbers);
        assert!(generator.include_page_numbers);
    }

    #[test]
    fn test_file_content_creation() {
        let content = FileContent::new(
            "test.rs".to_string(),
            "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
        );
        
        assert_eq!(content.path, "test.rs");
        assert_eq!(content.lines.len(), 3);
        assert_eq!(content.lines[0].content, "fn main() {");
    }

    #[test]
    fn test_create_simple_pdf() -> Result<()> {
        let theme = Theme::light();
        let generator = PdfGenerator::new(theme, true, false);
        
        let file_content = FileContent::new(
            "test.rs".to_string(),
            "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
        );

        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_str().unwrap();
        
        generator.create_pdf(temp_path, vec![file_content])?;
        
        // Check that file was created and has some content
        let metadata = std::fs::metadata(temp_path)?;
        assert!(metadata.len() > 0);
        
        Ok(())
    }
}