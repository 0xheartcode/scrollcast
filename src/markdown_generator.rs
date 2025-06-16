use anyhow::{Context, Result};
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

    pub fn generate_markdown(&self, files: Vec<FileInfo>, repo_name: &str) -> Result<String> {
        let mut markdown = String::new();

        // Title and metadata
        markdown.push_str(&format!("# {}\n\n", repo_name));
        markdown.push_str(&format!("Generated on: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));

        // Table of contents
        if self.include_toc {
            markdown.push_str("## Table of Contents\n\n");
            for file in &files {
                let sanitized_path = file.path.replace(['/', '\\'], "-").replace('.', "-");
                markdown.push_str(&format!("- [{}](#{sanitized_path})\n", file.path));
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
        
        for file in files {
            let sanitized_path = file.path.replace(['/', '\\'], "-").replace('.', "-");
            markdown.push_str(&format!("### {} {{#{sanitized_path}}}\n\n", file.path));
            
            if let Some(language) = &file.language {
                markdown.push_str(&format!("```{}\n", language));
            } else {
                markdown.push_str("```\n");
            }
            
            markdown.push_str(&file.content);
            
            if !file.content.ends_with('\n') {
                markdown.push('\n');
            }
            
            markdown.push_str("```\n\n");
            
            // Add file info
            markdown.push_str(&format!("*File size: {} bytes*\n\n", file.size));
            markdown.push_str("---\n\n");
        }

        Ok(markdown)
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
            "sol" => "solidity",  // Solidity support
            "vy" => "python",     // Vyper (use python highlighting as fallback)
            "move" => "rust",     // Move language (use rust as fallback)
            _ => return None,
        };

        Some(language.to_string())
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