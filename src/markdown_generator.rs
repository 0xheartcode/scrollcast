use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct MarkdownGenerator {
    include_toc: bool,
    include_file_tree: bool,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub content: String,
    pub language: Option<String>,
    pub size: usize,
}

impl MarkdownGenerator {
    pub fn new(include_toc: bool, include_file_tree: bool) -> Self {
        Self {
            include_toc,
            include_file_tree,
        }
    }

    pub fn generate_markdown(&self, files: &[FileInfo], repo_name: &str) -> Result<String> {
        let mut markdown = String::new();

        // Title and metadata
        markdown.push_str(&format!("# {}\n\n", repo_name));
        markdown.push_str(&format!("Generated on: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        // Table of contents
        if self.include_toc {
            markdown.push_str("## Table of Contents\n\n");
            for file in files {
                let sanitized_path = file.path.replace(['/', '\\'], "-").replace('.', "-");
                let escaped_path = self.escape_markdown_special_chars(&file.path);
                markdown.push_str(&format!("- [{}](#{sanitized_path})\n", escaped_path));
            }
            markdown.push_str("\n");
        }

        // File tree
        if self.include_file_tree {
            markdown.push_str("## File Structure\n\n");
            markdown.push_str("```\n");
            markdown.push_str(&self.generate_file_tree(&files));
            markdown.push_str("```\n\n");
        }

        // File contents
        markdown.push_str("## File Contents\n\n");
        
        let _total_files = files.len();
        let mut _global_page_number = 1; // Start after title/TOC page
        
        for (file_index, file) in files.iter().enumerate() {
            let _file_counter = file_index + 1;
            _global_page_number += 1; // Each file gets a new page
            
            // Add page break before each file (except the first one)
            if file_index > 0 {
                markdown.push_str("\n\\newpage\n\n");
            }
            let sanitized_path = file.path.replace(['/', '\\'], "-").replace('.', "-");
            let escaped_path = self.escape_markdown_special_chars(&file.path);
            markdown.push_str(&format!("### {} {{#{sanitized_path}}}\n\n", escaped_path));
            markdown.push_str(&format!("**Size:** {}\n\n", MarkdownGenerator::format_file_size(file.size)));
            
            if let Some(language) = &file.language {
                markdown.push_str(&format!("```{}\n", language));
            } else {
                markdown.push_str("```\n");
            }
            
            // Process content to prevent LaTeX errors
            let processed_content = self.process_content_for_latex(&file.content);
            markdown.push_str(&processed_content);
            
            // Ensure there's always a newline before closing backticks
            if !processed_content.ends_with('\n') {
                markdown.push('\n');
            }
            
            markdown.push_str("```\n\n");
            markdown.push_str("---\n\n");
        }

        Ok(markdown)
    }

    pub fn format_file_size(size: usize) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
        let mut size_f = size as f64;
        let mut unit_index = 0;
        
        while size_f >= 1024.0 && unit_index < UNITS.len() - 1 {
            size_f /= 1024.0;
            unit_index += 1;
        }
        
        if unit_index == 0 {
            format!("{} {}", size, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size_f, UNITS[unit_index])
        }
    }

    fn generate_file_tree(&self, files: &[FileInfo]) -> String {
        let mut tree = String::new();
        let mut dirs: HashMap<String, Vec<&str>> = HashMap::new();
        
        // Group files by directory
        for file in files {
            let path = Path::new(&file.path);
            if let Some(parent) = path.parent() {
                let parent_str = parent.to_string_lossy().to_string();
                dirs.entry(parent_str).or_default().push(&file.path);
            } else {
                dirs.entry(".".to_string()).or_default().push(&file.path);
            }
        }

        // Simple tree representation
        for file in files {
            tree.push_str(&format!("{}\n", file.path));
        }

        tree
    }

    pub fn detect_language(file_path: &str) -> Option<String> {
        let path = Path::new(file_path);
        
        // Handle special cases first
        if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy();
            // Handle .env files and variants like .env.local, .env.production, etc.
            if file_name_str.starts_with(".env") {
                return Some("bash".to_string());
            }
            // Handle Dockerfile variants
            if file_name_str.to_lowercase().starts_with("dockerfile") {
                return Some("dockerfile".to_string());
            }
        }
        
        let extension = path.extension()?.to_str()?;
        
        let language = match extension.to_lowercase().as_str() {
            "rs" => "rust",
            "py" => "python", 
            "js" => "javascript",
            "ts" => "typescript",
            "jsx" => "jsx",
            "tsx" => "tsx",
            "html" | "htm" => "html",
            "css" => "css",
            "scss" | "sass" => "scss",
            "json" => "json",
            "xml" => "xml",
            "yml" | "yaml" => "yaml",
            "toml" => "toml",
            "md" | "markdown" => "markdown",
            "sh" | "bash" => "bash",
            "zsh" => "zsh",
            "fish" => "fish",
            "c" => "c",
            "cpp" | "cc" | "cxx" | "c++" => "cpp",
            "h" | "hpp" => "c",
            "go" => "go",
            "java" => "java",
            "kt" | "kts" => "kotlin",
            "swift" => "swift",
            "php" => "php",
            "rb" => "ruby",
            "pl" => "perl",
            "lua" => "lua",
            "r" => "r",
            "sql" => "sql",
            "dockerfile" => "dockerfile",
            "env" => "bash",  // Environment files use shell-like syntax
            "sol" => "solidity",
            "vy" => "python",     // Vyper (use python highlighting as fallback)
            "move" => "rust",     // Move language (use rust as fallback)
            _ => return None,
        };

        Some(language.to_string())
    }

    fn escape_markdown_special_chars(&self, text: &str) -> String {
        // Escape characters that have special meaning in markdown/LaTeX outside code blocks
        text.replace('_', "\\_")     // Escape underscores that could be interpreted as emphasis
            .replace('#', "\\#")     // Escape hash symbols
            .replace('$', "\\$")     // Escape dollar signs (LaTeX math mode)
            .replace('%', "\\%")     // Escape percent signs (LaTeX comments)
            .replace('&', "\\&")     // Escape ampersands
            .replace('^', "\\^")     // Escape carets
            .replace('{', "\\{")     // Escape curly braces
            .replace('}', "\\}")
    }

    fn process_content_for_latex(&self, content: &str) -> String {
        // Break very long lines to prevent LaTeX "dimension too large" errors
        let lines: Vec<&str> = content.lines().collect();
        let mut processed_lines = Vec::new();
        
        for line in lines {
            if line.len() > 100 {
                // Break long lines at reasonable breakpoints
                let mut current_line = String::new();
                let chars: Vec<char> = line.chars().collect();
                
                for (_i, &ch) in chars.iter().enumerate() {
                    current_line.push(ch);
                    
                    // Break at 100 characters or at natural breakpoints
                    if current_line.len() >= 100 && (ch == ' ' || ch == ',' || ch == ';' || ch == ')' || ch == '}') {
                        processed_lines.push(current_line.clone());
                        current_line.clear();
                    }
                }
                
                // Add remaining characters
                if !current_line.is_empty() {
                    processed_lines.push(current_line);
                }
            } else {
                processed_lines.push(line.to_string());
            }
        }
        
        processed_lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert_eq!(MarkdownGenerator::detect_language("main.rs"), Some("rust".to_string()));
        assert_eq!(MarkdownGenerator::detect_language("script.py"), Some("python".to_string()));
        assert_eq!(MarkdownGenerator::detect_language("contract.sol"), Some("solidity".to_string()));
        assert_eq!(MarkdownGenerator::detect_language("unknown.xyz"), None);
    }

    #[test]
    fn test_markdown_generation() {
        let generator = MarkdownGenerator::new(true, true);
        let files = vec![
            FileInfo {
                path: "main.rs".to_string(),
                content: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
                language: Some("rust".to_string()),
                size: 44,
            }
        ];

        let markdown = generator.generate_markdown(files, "test-repo").unwrap();
        assert!(markdown.contains("# test-repo"));
        assert!(markdown.contains("## Table of Contents"));
        assert!(markdown.contains("## File Structure"));
        assert!(markdown.contains("### main.rs"));
        assert!(markdown.contains("```rust"));
    }
}