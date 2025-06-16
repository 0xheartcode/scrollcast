use anyhow::{anyhow, Context, Result};
use std::process::Command;
use std::path::{Path, PathBuf};
use std::fs;
use dirs;

#[derive(Debug, Clone)]
pub struct PandocConfig {
    pub output_format: OutputFormat,
    pub highlight_style: String,
    pub include_toc: bool,
    pub syntax_definitions: Vec<PathBuf>,
}

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

pub struct PandocConverter {
    config: PandocConfig,
    syntax_dir: PathBuf,
}

impl PandocConverter {
    pub fn new(config: PandocConfig) -> Result<Self> {
        let syntax_dir = Self::get_syntax_dir()?;
        fs::create_dir_all(&syntax_dir)
            .context("Failed to create syntax definitions directory")?;

        Ok(Self {
            config,
            syntax_dir,
        })
    }

    pub async fn setup_solidity_support(&self) -> Result<PathBuf> {
        let solidity_xml_path = self.syntax_dir.join("solidity.xml");
        
        if !solidity_xml_path.exists() {
            println!("📥 Downloading Solidity syntax definition...");
            self.download_solidity_definition(&solidity_xml_path).await?;
            println!("✅ Solidity syntax definition downloaded");
        }

        Ok(solidity_xml_path)
    }

    async fn download_solidity_definition(&self, output_path: &Path) -> Result<()> {
        let url = "https://raw.githubusercontent.com/KDE/syntax-highlighting/master/data/syntax/solidity.xml";
        
        let response = reqwest::get(url)
            .await
            .context("Failed to download Solidity syntax definition")?;
        
        let content = response.text()
            .await
            .context("Failed to read Solidity syntax definition content")?;
        
        fs::write(output_path, content)
            .context("Failed to write Solidity syntax definition to file")?;

        Ok(())
    }

    pub async fn convert_markdown_to_document(&self, input_path: &Path, output_path: &Path) -> Result<()> {
        // Ensure Solidity support is set up
        let solidity_xml = self.setup_solidity_support().await?;

        // Check if pandoc is available
        self.check_pandoc_available()?;

        // Build pandoc command
        let mut cmd = Command::new("pandoc");
        
        // Input file
        cmd.arg(input_path);
        
        // Output file
        cmd.arg("-o").arg(output_path);
        
        // Highlight style
        cmd.arg("--highlight-style").arg(&self.config.highlight_style);
        
        // Skip broken Solidity syntax definition for now
        // cmd.arg("--syntax-definition").arg(&solidity_xml);
        for syntax_def in &self.config.syntax_definitions {
            cmd.arg("--syntax-definition").arg(syntax_def);
        }
        
        // Format-specific options
        match self.config.output_format {
            OutputFormat::Pdf => {
                cmd.arg("--pdf-engine=xelatex");
                // Add line wrapping and formatting options
                cmd.arg("--wrap=preserve");
                cmd.arg("-V").arg("geometry:margin=1in");
                cmd.arg("-V").arg("fontsize=10pt");
                // Don't force monospace font to allow syntax highlighting themes
                if self.config.include_toc {
                    cmd.arg("--toc");
                }
            },
            OutputFormat::Epub => {
                // Add line wrapping for EPUB  
                cmd.arg("--wrap=preserve");
                // Only add minimal CSS that doesn't interfere with syntax highlighting
                let temp_css = self.create_minimal_epub_css_file()?;
                cmd.arg("--css").arg(&temp_css);
                if self.config.include_toc {
                    cmd.arg("--toc");
                }
                // Could add cover image support later
            },
            OutputFormat::Html => {
                cmd.arg("--standalone");
                // Only add minimal CSS that doesn't interfere with syntax highlighting
                let temp_css = self.create_minimal_html_css_file()?;
                cmd.arg("--css").arg(&temp_css);
                if self.config.include_toc {
                    cmd.arg("--toc");
                }
            },
            OutputFormat::Markdown => {
                // For markdown output, we can just copy the generated markdown file
                // But we'll still use pandoc for consistency and potential processing
                if self.config.include_toc {
                    cmd.arg("--toc");
                }
            },
        }

        println!("🔄 Converting with Pandoc...");
        println!("📝 Command: {:?}", cmd);
        
        // Execute pandoc
        let output = cmd.output()
            .context("Failed to execute pandoc command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(anyhow!(
                "Pandoc conversion failed:\nSTDERR: {}\nSTDOUT: {}", 
                stderr, stdout
            ));
        }

        println!("✅ Document generated successfully: {}", output_path.display());
        Ok(())
    }

    fn check_pandoc_available(&self) -> Result<()> {
        let output = Command::new("pandoc")
            .arg("--version")
            .output()
            .context("Failed to check pandoc version. Is pandoc installed?")?;

        if !output.status.success() {
            return Err(anyhow!("Pandoc is not available or not working properly"));
        }

        let version_info = String::from_utf8_lossy(&output.stdout);
        println!("📄 Using {}", version_info.lines().next().unwrap_or("pandoc"));
        
        Ok(())
    }

    fn get_syntax_dir() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .or_else(|| dirs::home_dir().map(|p| p.join(".local/share")))
            .context("Could not determine local data directory")?;
        
        Ok(data_dir.join("scrollcast").join("syntax"))
    }

    fn create_minimal_epub_css_file(&self) -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir();
        let css_path = temp_dir.join("scrollcast_epub_minimal.css");
        
        let css_content = r#"
/* Ultra-minimal CSS for EPUB that doesn't interfere with syntax highlighting */
pre {
    white-space: pre-wrap;
    word-wrap: break-word;
    overflow-wrap: break-word;
}
"#;
        
        fs::write(&css_path, css_content)
            .context("Failed to create minimal EPUB CSS file")?;
        
        Ok(css_path)
    }

    fn create_minimal_html_css_file(&self) -> Result<PathBuf> {
        let temp_dir = std::env::temp_dir();
        let css_path = temp_dir.join("scrollcast_html_minimal.css");
        
        let css_content = r#"
/* Ultra-minimal CSS that doesn't interfere with Pandoc syntax highlighting */
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    line-height: 1.6;
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
}

/* Only add line wrapping, no colors or backgrounds */
pre {
    white-space: pre-wrap;
    word-wrap: break-word;
    overflow-wrap: break-word;
    overflow-x: auto;
}
"#;
        
        fs::write(&css_path, css_content)
            .context("Failed to create minimal HTML CSS file")?;
        
        Ok(css_path)
    }

    pub fn list_available_highlight_styles() -> Result<Vec<String>> {
        let output = Command::new("pandoc")
            .arg("--list-highlight-styles")
            .output()
            .context("Failed to get highlight styles from pandoc")?;

        if !output.status.success() {
            return Err(anyhow!("Failed to list pandoc highlight styles"));
        }

        let styles = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(styles)
    }

    pub fn list_available_languages() -> Result<Vec<String>> {
        let output = Command::new("pandoc")
            .arg("--list-highlight-languages")
            .output()
            .context("Failed to get supported languages from pandoc")?;

        if !output.status.success() {
            return Err(anyhow!("Failed to list pandoc supported languages"));
        }

        let languages = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(languages)
    }
}

impl Default for PandocConfig {
    fn default() -> Self {
        Self {
            output_format: OutputFormat::Pdf,
            highlight_style: "kate".to_string(),
            include_toc: true,
            syntax_definitions: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_output_format_extension() {
        assert_eq!(OutputFormat::Pdf.extension(), "pdf");
        assert_eq!(OutputFormat::Epub.extension(), "epub");
        assert_eq!(OutputFormat::Html.extension(), "html");
    }

    #[test]
    fn test_pandoc_config_default() {
        let config = PandocConfig::default();
        assert_eq!(config.highlight_style, "kate");
        assert!(config.include_toc);
        assert!(matches!(config.output_format, OutputFormat::Pdf));
    }
}