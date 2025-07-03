use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use toml;

use crate::theme::{ThemeMode, ColorScheme};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub theme: ThemeConfig,
    #[serde(default)]
    pub formatting: FormattingConfig,
    #[serde(default)]
    pub ignore: IgnoreConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub folder: String,
    pub filename: Option<String>,
    pub create_folder: bool,
    pub single_file: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub mode: String, // "light" or "dark"
    pub font_size: f32,
    pub line_height: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_colors: Option<ColorScheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattingConfig {
    pub line_numbers: bool,
    pub page_numbers: bool,
    pub syntax_highlighting: bool,
    pub remove_comments: bool,
    pub remove_empty_lines: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreConfig {
    pub files: Vec<String>,
    pub extensions: Vec<String>,
    pub directories: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output: OutputConfig::default(),
            theme: ThemeConfig::default(),
            formatting: FormattingConfig::default(),
            ignore: IgnoreConfig::default(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            folder: "output".to_string(),
            filename: None,
            create_folder: true,
            single_file: true,
        }
    }
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            mode: "light".to_string(),
            font_size: 10.0,
            line_height: 1.2,
            custom_colors: None,
        }
    }
}

impl Default for FormattingConfig {
    fn default() -> Self {
        Self {
            line_numbers: true,
            page_numbers: true,
            syntax_highlighting: true,
            remove_comments: false,
            remove_empty_lines: false,
        }
    }
}

impl Default for IgnoreConfig {
    fn default() -> Self {
        Self {
            files: vec![],
            extensions: vec![],
            directories: vec![],
        }
    }
}

impl Config {
    /// Load configuration from a file, falling back to defaults if not found
    #[allow(dead_code)]
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config_path = path.as_ref();
        
        if config_path.exists() {
            let content = fs::read_to_string(config_path)
                .context("Failed to read configuration file")?;
            let config: Config = toml::from_str(&content)
                .context("Failed to parse configuration file")?;
            Ok(config)
        } else {
            Ok(Config::default())
        }
    }

    /// Load configuration from the current directory or user's home directory
    #[allow(dead_code)]
    pub fn load_default() -> Result<Self> {
        // Try to load from current directory first
        let local_config = Path::new("scrollcast.toml");
        if local_config.exists() {
            return Self::load_from_file(local_config);
        }

        // Try to load from home directory
        if let Some(home_dir) = dirs::home_dir() {
            let global_config = home_dir.join(".scrollcast.toml");
            if global_config.exists() {
                return Self::load_from_file(global_config);
            }
        }

        // Fall back to defaults
        Ok(Config::default())
    }

    /// Save configuration to a file
    #[allow(dead_code)]
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize configuration")?;
        fs::write(path, content)
            .context("Failed to write configuration file")?;
        Ok(())
    }

    /// Create a sample configuration file
    #[allow(dead_code)]
    pub fn create_sample_config<P: AsRef<Path>>(path: P) -> Result<()> {
        let sample_config = Config {
            output: OutputConfig {
                folder: "output".to_string(),
                filename: Some("repository.pdf".to_string()),
                create_folder: true,
                single_file: true,
            },
            theme: ThemeConfig {
                mode: "light".to_string(),
                font_size: 10.0,
                line_height: 1.2,
                custom_colors: None,
            },
            formatting: FormattingConfig {
                line_numbers: true,
                page_numbers: true,
                syntax_highlighting: true,
                remove_comments: false,
                remove_empty_lines: false,
            },
            ignore: IgnoreConfig {
                files: vec![
                    "*.tmp".to_string(),
                    "*.log".to_string(),
                    ".env".to_string(),
                ],
                extensions: vec![
                    ".tmp".to_string(),
                    ".log".to_string(),
                    ".cache".to_string(),
                ],
                directories: vec![
                    "tmp".to_string(),
                    "temp".to_string(),
                    "logs".to_string(),
                ],
            },
        };

        sample_config.save_to_file(path)?;
        Ok(())
    }

    /// Get theme mode as enum
    #[allow(dead_code)]
    pub fn get_theme_mode(&self) -> ThemeMode {
        match self.theme.mode.as_str() {
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::Light,
        }
    }

    /// Get output directory path
    #[allow(dead_code)]
    pub fn get_output_dir(&self) -> PathBuf {
        PathBuf::from(&self.output.folder)
    }

    /// Get output filename with fallback
    #[allow(dead_code)]
    pub fn get_output_filename(&self, fallback: &str) -> String {
        self.output.filename
            .as_ref()
            .unwrap_or(&fallback.to_string())
            .clone()
    }

    /// Ensure output directory exists
    #[allow(dead_code)]
    pub fn ensure_output_dir(&self) -> Result<PathBuf> {
        let output_dir = self.get_output_dir();
        
        if self.output.create_folder && !output_dir.exists() {
            fs::create_dir_all(&output_dir)
                .context("Failed to create output directory")?;
            println!("📁 Created output directory: {}", output_dir.display());
        }
        
        Ok(output_dir)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.output.folder, "output");
        assert_eq!(config.theme.mode, "light");
        assert!(config.formatting.line_numbers);
    }

    #[test]
    fn test_config_serialization() -> Result<()> {
        let config = Config::default();
        let toml_str = toml::to_string(&config)?;
        let parsed: Config = toml::from_str(&toml_str)?;
        
        assert_eq!(config.output.folder, parsed.output.folder);
        assert_eq!(config.theme.mode, parsed.theme.mode);
        
        Ok(())
    }

    #[test]
    fn test_config_file_operations() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let config_path = temp_dir.path().join("test.toml");
        
        // Create and save config
        let config = Config::default();
        config.save_to_file(&config_path)?;
        
        // Load config back
        let loaded_config = Config::load_from_file(&config_path)?;
        assert_eq!(config.output.folder, loaded_config.output.folder);
        
        Ok(())
    }

    #[test]
    fn test_output_directory_creation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let output_folder = temp_dir.path().join("test_output");
        
        let config = Config {
            output: OutputConfig {
                folder: output_folder.to_string_lossy().to_string(),
                create_folder: true,
                ..Default::default()
            },
            ..Default::default()
        };

        let created_dir = config.ensure_output_dir()?;
        assert!(created_dir.exists());
        assert!(created_dir.is_dir());
        
        Ok(())
    }
}