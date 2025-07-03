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
    pub use_chunked_pdf: bool,
}

#[derive(Debug, Clone)]
pub enum OutputFormat {
    Pdf,
    Epub,
    Html,
    Markdown,
}

impl OutputFormat {
    #[allow(dead_code)]
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

    pub async fn convert_markdown_to_document(&self, input_path: &Path, output_path: &Path, verbose: bool) -> Result<()> {
        // Ensure Solidity support is set up
        let _solidity_xml = self.setup_solidity_support().await?;

        // Check if pandoc is available
        self.check_pandoc_available()?;

        // Build pandoc command with resource limits
        // Use timeout and ulimit to prevent resource exhaustion
        let timeout_seconds = std::env::var("SCROLLCAST_PANDOC_TIMEOUT")
            .unwrap_or_else(|_| "600".to_string()); // 10 minutes default
        let memory_limit_kb = std::env::var("SCROLLCAST_PANDOC_MEMORY_KB")
            .unwrap_or_else(|_| "4194304".to_string()); // 4GB default
        let cpu_time_seconds = std::env::var("SCROLLCAST_PANDOC_CPU_TIME")
            .unwrap_or_else(|_| "600".to_string()); // 10 minutes default
            
        let mut cmd = Command::new("timeout");
        cmd.arg(&timeout_seconds)
            .arg("bash")
            .arg("-c")
            .arg(&format!("ulimit -m {} && ulimit -t {} && exec pandoc \"$@\"", 
                memory_limit_kb, cpu_time_seconds))
            .arg("--");
        
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
                // Add line wrapping and formatting options for large code blocks
                cmd.arg("--wrap=auto");
                cmd.arg("-V").arg("geometry:margin=0.8in");
                cmd.arg("-V").arg("fontsize=9pt");
                cmd.arg("-V").arg("linestretch=1.1");
                // Fix for large code blocks causing "dimension too large" errors
                cmd.arg("-V").arg("documentclass=article");
                cmd.arg("-V").arg("pagestyle=plain");
                // Use default code block handling instead of listings package to avoid spacing issues
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
        if verbose {
            println!("📝 Command: {:?}", cmd);
        }
        
        // Execute pandoc
        let output = cmd.output()
            .context("Failed to execute pandoc command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // Check for specific timeout/resource limit issues
            let exit_code = output.status.code().unwrap_or(-1);
            let error_message = match exit_code {
                124 => "Pandoc conversion timed out. Try reducing document size or increasing SCROLLCAST_PANDOC_TIMEOUT.",
                137 => "Pandoc killed due to memory limit. Try reducing document size or increasing SCROLLCAST_PANDOC_MEMORY_KB.",
                _ if stderr.contains("TeX capacity exceeded") || stderr.contains("dimension too large") => {
                    "LaTeX capacity exceeded. Document is too large for PDF generation. Try EPUB or HTML format instead."
                },
                _ if stderr.contains("memory") || stderr.contains("out of memory") => {
                    "Out of memory error. Try reducing document size or increasing SCROLLCAST_PANDOC_MEMORY_KB."
                },
                _ => "Pandoc conversion failed"
            };
            
            return Err(anyhow!(
                "{}:\nExit code: {}\nSTDERR: {}\nSTDOUT: {}", 
                error_message, exit_code, stderr, stdout
            ));
        }

        println!("✅ Document generated successfully: {}", output_path.display());
        Ok(())
    }

    pub async fn convert_markdown_chunks_to_pdf(&self, markdown_files: &[PathBuf], output_path: &Path, verbose: bool) -> Result<()> {
        if !matches!(self.config.output_format, OutputFormat::Pdf) {
            return Err(anyhow!("Chunked processing is only supported for PDF output"));
        }

        // Check if PDF merge tools are available
        self.check_pdf_merge_tools()?;

        println!("📄 Converting {} markdown chunks to individual PDFs...", markdown_files.len());
        
        let temp_dir = std::env::temp_dir();
        let mut temp_pdfs = Vec::new();
        
        // Convert each markdown file to PDF individually
        for (index, markdown_file) in markdown_files.iter().enumerate() {
            if verbose {
                println!("   📄 Converting chunk {}/{}: {}", 
                    index + 1, markdown_files.len(), markdown_file.display());
            }
            
            let temp_pdf = temp_dir.join(format!("scrollcast_chunk_{}.pdf", index));
            self.convert_single_markdown_to_pdf(markdown_file, &temp_pdf, verbose).await?;
            temp_pdfs.push(temp_pdf);
        }
        
        // Merge all PDFs into final output
        println!("🔗 Merging {} PDFs into final document...", temp_pdfs.len());
        self.merge_pdfs(&temp_pdfs, output_path, verbose)?;
        
        // Clean up temporary PDFs
        for temp_pdf in &temp_pdfs {
            let _ = fs::remove_file(temp_pdf);
        }
        
        println!("✅ Document generated successfully: {}", output_path.display());
        Ok(())
    }

    async fn convert_single_markdown_to_pdf(&self, input_path: &Path, output_path: &Path, verbose: bool) -> Result<()> {
        // Use the same resource limits but for individual files
        let timeout_seconds = std::env::var("SCROLLCAST_PANDOC_TIMEOUT")
            .unwrap_or_else(|_| "300".to_string()); // 5 minutes for individual files
        let memory_limit_kb = std::env::var("SCROLLCAST_PANDOC_MEMORY_KB")
            .unwrap_or_else(|_| "2097152".to_string()); // 2GB for individual files
        let cpu_time_seconds = std::env::var("SCROLLCAST_PANDOC_CPU_TIME")
            .unwrap_or_else(|_| "300".to_string()); // 5 minutes for individual files
            
        let mut cmd = Command::new("timeout");
        cmd.arg(&timeout_seconds)
            .arg("bash")
            .arg("-c")
            .arg(&format!("ulimit -m {} && ulimit -t {} && exec pandoc \"$@\"", 
                memory_limit_kb, cpu_time_seconds))
            .arg("--")
            .arg(input_path)
            .arg("-o").arg(output_path)
            .arg("--highlight-style").arg(&self.config.highlight_style)
            .arg("--pdf-engine=xelatex")
            .arg("--wrap=auto")
            .arg("-V").arg("geometry:margin=0.8in")
            .arg("-V").arg("fontsize=9pt")
            .arg("-V").arg("linestretch=1.1")
            .arg("-V").arg("documentclass=article")
            .arg("-V").arg("pagestyle=plain");

        if verbose {
            println!("📝 Command: {:?}", cmd);
        }
        
        let output = cmd.output()
            .context("Failed to execute pandoc command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(anyhow!(
                "Pandoc conversion failed for {}:\nSTDERR: {}\nSTDOUT: {}", 
                input_path.display(), stderr, stdout
            ));
        }

        Ok(())
    }

    fn merge_pdfs(&self, pdf_files: &[PathBuf], output_path: &Path, verbose: bool) -> Result<()> {
        // Try pdfunite first (faster), then pdftk as fallback
        if self.try_pdfunite(pdf_files, output_path, verbose).is_ok() {
            return Ok(());
        }
        
        println!("📄 pdfunite failed, trying pdftk...");
        self.try_pdftk(pdf_files, output_path, verbose)
    }

    fn try_pdfunite(&self, pdf_files: &[PathBuf], output_path: &Path, verbose: bool) -> Result<()> {
        let mut cmd = Command::new("pdfunite");
        
        for pdf_file in pdf_files {
            cmd.arg(pdf_file);
        }
        cmd.arg(output_path);

        if verbose {
            println!("📝 pdfunite command: {:?}", cmd);
        }

        let output = cmd.output()
            .context("Failed to execute pdfunite")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("pdfunite failed: {}", stderr));
        }

        Ok(())
    }

    fn try_pdftk(&self, pdf_files: &[PathBuf], output_path: &Path, verbose: bool) -> Result<()> {
        let mut cmd = Command::new("pdftk");
        
        for pdf_file in pdf_files {
            cmd.arg(pdf_file);
        }
        cmd.arg("cat").arg("output").arg(output_path);

        if verbose {
            println!("📝 pdftk command: {:?}", cmd);
        }

        let output = cmd.output()
            .context("Failed to execute pdftk")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("pdftk failed: {}", stderr));
        }

        Ok(())
    }

    fn check_pdf_merge_tools(&self) -> Result<()> {
        // Check if either pdfunite or pdftk is available
        let pdfunite_available = Command::new("pdfunite")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let pdftk_available = Command::new("pdftk")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !pdfunite_available && !pdftk_available {
            return Err(anyhow!(
                "PDF merging requires either 'pdfunite' (from poppler-utils) or 'pdftk'. \
                Please install one of these tools to use chunked PDF processing."
            ));
        }

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
            use_chunked_pdf: false,
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