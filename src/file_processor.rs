use anyhow::{Context, Result};
use content_inspector::{inspect, ContentType};
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use ignore::WalkBuilder;

use crate::markdown_generator::{FileInfo, MarkdownGenerator};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreConfig {
    pub ignored_files: Vec<String>,
    pub ignored_extensions: Vec<String>,
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            ignored_files: vec![],
            ignored_extensions: vec![],
        }
    }
}

pub struct FileProcessor {
    ignore_config: IgnoreConfig,
    universal_excludes: UniversalExcludes,
    respect_gitignore: bool,
    ignored_directories: Vec<String>,
}

impl FileProcessor {
    pub fn new() -> Self {
        Self {
            ignore_config: IgnoreConfig::default(),
            universal_excludes: UniversalExcludes::new(),
            respect_gitignore: true,
            ignored_directories: Vec::new(),
        }
    }

    pub fn with_ignore_config(mut self, config: IgnoreConfig) -> Self {
        self.ignore_config = config;
        self
    }

    pub fn with_gitignore_respect(mut self, respect: bool) -> Self {
        self.respect_gitignore = respect;
        self
    }

    pub fn with_ignored_directories(mut self, dirs: Vec<String>) -> Self {
        self.ignored_directories = dirs;
        self
    }

    pub fn load_ignore_config_from_path<P: AsRef<Path>>(mut self, path: P) -> Result<Self> {
        let ignore_file_path = path.as_ref().join("scrollcast.ignore");
        if ignore_file_path.exists() {
            let content = fs::read_to_string(&ignore_file_path)
                .context("Failed to read ignore configuration file")?;
            let config: IgnoreConfig = serde_json::from_str(&content)
                .context("Failed to parse ignore configuration file")?;
            self.ignore_config = config;
        }
        Ok(self)
    }

    pub fn process_directory<P: AsRef<Path>>(&self, path: P) -> Result<Vec<FileInfo>> {
        let mut files = Vec::new();
        let root_path = path.as_ref();

        // Use ignore crate for proper gitignore handling
        let walker = if self.respect_gitignore {
            WalkBuilder::new(root_path)
                .git_ignore(true)
                .git_global(true)
                .git_exclude(true)
                .hidden(false)
                .follow_links(false)
                .build()
        } else {
            WalkBuilder::new(root_path)
                .git_ignore(false)
                .git_global(false)
                .git_exclude(false)
                .hidden(false)
                .follow_links(false)
                .build()
        };

        for result in walker {
            match result {
                Ok(entry) => {
                    if entry.file_type().map_or(false, |ft| ft.is_file()) {
                        let file_path = entry.path();
                        
                        if self.should_process_file_simple(file_path, root_path)? {
                            match self.process_single_file(file_path, root_path) {
                                Ok(file_info) => files.push(file_info),
                                Err(e) => {
                                    eprintln!("Warning: Failed to process file {}: {}", file_path.display(), e);
                                    continue;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read directory entry: {}", e);
                    continue;
                }
            }
        }

        // Show warning for large file counts
        if files.len() > 50 {
            eprintln!("⚠️  Warning: Processing {} files. This may take a while and result in a large document.", files.len());
            
            // Show top directories by file count
            let dir_counts = self.get_directory_file_counts(&files);
            if !dir_counts.is_empty() {
                eprintln!("   Top directories by file count:");
                for (dir, count) in dir_counts.iter().take(5) {
                    eprintln!("     {} - {} files", dir, count);
                }
            }
            
            eprintln!("   Consider using .gitignore or custom ignore rules to reduce the number of files.");
        }

        // Sort files by path for consistent output
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(files)
    }

    fn should_process_file_simple(&self, file_path: &Path, root_path: &Path) -> Result<bool> {
        // Get relative path for checking
        let relative_path = file_path.strip_prefix(root_path)
            .context("Failed to get relative path")?;
        let relative_path_str = relative_path.to_string_lossy();

        // Check user-specified ignored directories
        for ignored_dir in &self.ignored_directories {
            if relative_path_str.starts_with(ignored_dir) || 
               relative_path_str.starts_with(&format!("{}/", ignored_dir)) {
                return Ok(false);
            }
        }

        // Check universal excludes
        if self.universal_excludes.should_exclude(file_path) {
            return Ok(false);
        }

        // Check custom ignore configuration
        if self.ignore_config.ignored_files.iter().any(|ignored| {
            relative_path_str.contains(ignored) || 
            file_path.file_name().map_or(false, |name| name.to_string_lossy().contains(ignored))
        }) {
            return Ok(false);
        }

        // Check file extensions
        if let Some(extension) = file_path.extension() {
            let ext_str = format!(".{}", extension.to_string_lossy());
            if self.ignore_config.ignored_extensions.contains(&ext_str) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn get_directory_file_counts(&self, files: &[FileInfo]) -> Vec<(String, usize)> {
        use std::collections::HashMap;
        
        let mut dir_counts: HashMap<String, usize> = HashMap::new();
        
        for file in files {
            let path = Path::new(&file.path);
            let dir = if let Some(parent) = path.parent() {
                if parent == Path::new("") {
                    ".".to_string()
                } else {
                    parent.to_string_lossy().to_string()
                }
            } else {
                ".".to_string()
            };
            
            *dir_counts.entry(dir).or_insert(0) += 1;
        }
        
        // Sort by count descending
        let mut sorted: Vec<(String, usize)> = dir_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        
        sorted
    }

    fn process_single_file(&self, file_path: &Path, root_path: &Path) -> Result<FileInfo> {
        let relative_path = file_path.strip_prefix(root_path)
            .context("Failed to get relative path")?;
        let relative_path_str = relative_path.to_string_lossy().to_string();

        // Read file content
        let content = fs::read(file_path)
            .context(format!("Failed to read file: {}", file_path.display()))?;

        let file_size = content.len();

        // Check if file is binary
        let (text_content, detected_language) = match inspect(&content) {
            ContentType::BINARY => {
                // For binary files, we'll include a placeholder
                let placeholder = format!("[Binary file: {} ({} bytes)]", 
                    file_path.file_name().unwrap_or_default().to_string_lossy(),
                    content.len()
                );
                (placeholder, None)
            }
            ContentType::UTF_8 | ContentType::UTF_8_BOM => {
                // Convert to string and detect language
                let text = String::from_utf8_lossy(&content).to_string();
                let language = MarkdownGenerator::detect_language(&relative_path_str);
                (text, language)
            }
            ContentType::UTF_16LE | ContentType::UTF_16BE | 
            ContentType::UTF_32LE | ContentType::UTF_32BE => {
                // Handle UTF-16/32 files
                let text = String::from_utf8_lossy(&content).to_string();
                let language = MarkdownGenerator::detect_language(&relative_path_str);
                (text, language)
            }
        };

        Ok(FileInfo {
            path: relative_path_str,
            content: text_content,
            language: detected_language,
            size: file_size,
        })
    }
}

pub struct UniversalExcludes {
    excluded_dirs: Vec<String>,
    excluded_files: Vec<String>,
    excluded_extensions: Vec<String>,
}

impl UniversalExcludes {
    pub fn new() -> Self {
        Self {
            excluded_dirs: vec![
                // Version control
                ".git".to_string(),
                ".svn".to_string(),
                ".hg".to_string(),
                
                // IDE and editor files
                ".vscode".to_string(),
                ".idea".to_string(),
                ".vs".to_string(),
                
                // Build directories
                "target".to_string(),
                "dist".to_string(),
                "build".to_string(),
                "out".to_string(),
                
                // Dependencies
                "node_modules".to_string(),
                "vendor".to_string(),
                ".cargo".to_string(),
                
                // Cache directories
                ".cache".to_string(),
                "__pycache__".to_string(),
                ".pytest_cache".to_string(),
                
                // OS files
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
            ],
            excluded_files: vec![
                // Version control
                ".gitignore".to_string(),
                ".gitmodules".to_string(),
                ".gitattributes".to_string(),
                
                // Package managers
                "package-lock.json".to_string(),
                "yarn.lock".to_string(),
                "Cargo.lock".to_string(),
                "composer.lock".to_string(),
                "Gemfile.lock".to_string(),
                
                // IDE files
                ".editorconfig".to_string(),
                
                // OS files
                ".DS_Store".to_string(),
                "Thumbs.db".to_string(),
                "desktop.ini".to_string(),
            ],
            excluded_extensions: vec![
                // Images
                ".png".to_string(), ".jpg".to_string(), ".jpeg".to_string(), ".gif".to_string(),
                ".bmp".to_string(), ".tiff".to_string(), ".svg".to_string(), ".ico".to_string(),
                ".webp".to_string(),
                
                // Videos
                ".mp4".to_string(), ".avi".to_string(), ".mov".to_string(), ".wmv".to_string(),
                ".flv".to_string(), ".webm".to_string(), ".mkv".to_string(),
                
                // Audio
                ".mp3".to_string(), ".wav".to_string(), ".flac".to_string(), ".aac".to_string(),
                ".ogg".to_string(), ".wma".to_string(),
                
                // Archives
                ".zip".to_string(), ".tar".to_string(), ".gz".to_string(), ".bz2".to_string(),
                ".rar".to_string(), ".7z".to_string(), ".xz".to_string(),
                
                // Executables
                ".exe".to_string(), ".dll".to_string(), ".so".to_string(), ".dylib".to_string(),
                ".bin".to_string(), ".app".to_string(),
                
                // Documents (binary formats)
                ".pdf".to_string(), ".doc".to_string(), ".docx".to_string(), ".xls".to_string(),
                ".xlsx".to_string(), ".ppt".to_string(), ".pptx".to_string(), ".ps".to_string(),
                
                // Fonts
                ".ttf".to_string(), ".otf".to_string(), ".woff".to_string(), ".woff2".to_string(),
                ".eot".to_string(),
            ],
        }
    }

    pub fn should_exclude(&self, path: &Path) -> bool {
        // Check if any parent directory should be excluded
        for component in path.components() {
            if let Some(name) = component.as_os_str().to_str() {
                if self.excluded_dirs.contains(&name.to_string()) {
                    return true;
                }
            }
        }

        // Check file name
        if let Some(file_name) = path.file_name() {
            let file_name_str = file_name.to_string_lossy().to_string();
            if self.excluded_files.contains(&file_name_str) {
                return true;
            }
        }

        // Check file extension
        if let Some(extension) = path.extension() {
            let ext_str = format!(".{}", extension.to_string_lossy());
            if self.excluded_extensions.contains(&ext_str) {
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_universal_excludes() {
        let excludes = UniversalExcludes::new();
        
        assert!(excludes.should_exclude(Path::new(".git/config")));
        assert!(excludes.should_exclude(Path::new("src/main.rs.png")));
        assert!(excludes.should_exclude(Path::new("node_modules/package/index.js")));
        assert!(!excludes.should_exclude(Path::new("src/main.rs")));
    }

    #[test]
    fn test_ignore_config() -> Result<()> {
        let config = IgnoreConfig {
            ignored_files: vec!["test.txt".to_string()],
            ignored_extensions: vec![".tmp".to_string()],
        };

        let processor = FileProcessor::new().with_ignore_config(config);
        
        // This would require setting up test files to fully test
        assert!(processor.ignore_config.ignored_files.contains(&"test.txt".to_string()));
        
        Ok(())
    }

    #[test]
    fn test_file_processing() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // Create test files
        let mut rust_file = File::create(temp_path.join("test.rs"))?;
        writeln!(rust_file, "fn main() {{\n    println!(\"Hello, world!\");\n}}")?;

        let mut txt_file = File::create(temp_path.join("readme.txt"))?;
        writeln!(txt_file, "This is a test file.")?;

        // Create a directory that should be excluded
        fs::create_dir(temp_path.join(".git"))?;
        File::create(temp_path.join(".git/config"))?;

        let processor = FileProcessor::new();
        let files = processor.process_directory(temp_path)?;

        // Should process the .rs and .txt files but not the .git directory
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.path.contains("test.rs")));
        assert!(files.iter().any(|f| f.path.contains("readme.txt")));
        assert!(!files.iter().any(|f| f.path.contains(".git")));

        Ok(())
    }

    #[test]
    fn test_binary_file_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // Create a binary file (PNG header)
        let mut binary_file = File::create(temp_path.join("test.png"))?;
        binary_file.write_all(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A])?;

        let processor = FileProcessor::new();
        let files = processor.process_directory(temp_path)?;

        // PNG files should be excluded by universal excludes
        assert_eq!(files.len(), 0);

        Ok(())
    }
}