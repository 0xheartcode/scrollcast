mod theme;
mod pdf_generator;
mod file_processor;
mod syntax_highlighter;
mod config;

use anyhow::Result;
use clap::{Arg, Command};
use theme::{Theme, ThemeMode};
use pdf_generator::{PdfGenerator, FileContent};
use syntax_highlighter::SyntaxHighlighter;
use config::Config;
use std::path::Path;

fn main() -> Result<()> {
    let matches = Command::new("git-to-pdf")
        .version("0.1.0")
        .author("heartcode <0xheartcode@gmail.com>")
        .about("Convert Git repositories to beautifully formatted PDF documents")
        .arg(
            Arg::new("input")
                .help("Repository URL or local path")
                .required_unless_present("create-config")
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Output PDF file path")
                .default_value("repository.pdf"),
        )
        .arg(
            Arg::new("theme")
                .short('t')
                .long("theme")
                .value_name("THEME")
                .help("Theme to use (light or dark)")
                .default_value("light"),
        )
        .arg(
            Arg::new("line-numbers")
                .long("line-numbers")
                .help("Include line numbers in output")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("page-numbers")
                .long("page-numbers")
                .help("Include page numbers in output")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Use specific configuration file"),
        )
        .arg(
            Arg::new("create-config")
                .long("create-config")
                .help("Create a sample configuration file")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    // Handle --create-config flag
    if matches.get_flag("create-config") {
        Config::create_sample_config("git-to-pdf.toml")?;
        println!("✅ Created sample configuration file: git-to-pdf.toml");
        println!("You can now edit this file to customize your settings.");
        return Ok(());
    }

    // Load configuration
    let config = if let Some(config_file) = matches.get_one::<String>("config") {
        Config::load_from_file(config_file)?
    } else {
        Config::load_default()?
    };

    let _input = matches.get_one::<String>("input");
    let output = matches.get_one::<String>("output").unwrap();
    let theme_str = matches.get_one::<String>("theme").unwrap();
    
    // CLI args override config settings
    let line_numbers = matches.get_flag("line-numbers") || config.formatting.line_numbers;
    let page_numbers = matches.get_flag("page-numbers") || config.formatting.page_numbers;

    // Parse theme - CLI overrides config
    let theme_mode = if theme_str != "light" {
        match theme_str.as_str() {
            "dark" => ThemeMode::Dark,
            "light" => ThemeMode::Light,
            _ => {
                eprintln!("Invalid theme '{}'. Using config default.", theme_str);
                config.get_theme_mode()
            }
        }
    } else {
        config.get_theme_mode()
    };

    let mut theme = Theme::from_mode(theme_mode);
    
    // Apply custom font settings from config
    theme.font_size = config.theme.font_size;
    theme.line_height = config.theme.line_height;
    println!("🎨 Using {} theme", match theme.mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
    });

    // Create syntax highlighter and sample content with proper highlighting
    let highlighter = SyntaxHighlighter::new(theme.clone());
    let sample_content = create_sample_content_with_highlighting(&highlighter);
    
    // Ensure output directory exists
    let output_dir = config.ensure_output_dir()?;
    
    // Determine final output path
    let final_output_path = if output == "repository.pdf" {
        // Use config filename if output wasn't specified
        output_dir.join(config.get_output_filename("repository.pdf"))
    } else {
        // Use specified output path
        if Path::new(output).is_absolute() {
            output.into()
        } else {
            output_dir.join(output)
        }
    };

    let generator = PdfGenerator::new(theme, line_numbers, page_numbers);
    
    println!("📄 Generating PDF...");
    generator.create_pdf(final_output_path.to_str().unwrap(), vec![sample_content])?;
    
    println!("✅ PDF generated successfully: {}", final_output_path.display());
    
    Ok(())
}

fn create_sample_content_with_highlighting(highlighter: &SyntaxHighlighter) -> FileContent {
    let rust_code = r#"use std::collections::HashMap;
use anyhow::Result;

fn main() -> Result<()> {
    let mut map = HashMap::new();
    map.insert("key", "value");
    
    // Print greeting
    println!("Hello, world!");
    
    let numbers = vec![1, 2, 3, 4, 5];
    for number in numbers {
        if number % 2 == 0 {
            println!("{} is even", number);
        } else {
            println!("{} is odd", number);
        }
    }
    
    Ok(())
}"#;

    // Use syntax highlighting for the sample content
    let highlighted_lines = highlighter.highlight_simple(rust_code, "rs");
    FileContent::with_highlighting("src/main.rs".to_string(), highlighted_lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_content_creation() {
        let theme = Theme::light();
        let highlighter = SyntaxHighlighter::new(theme);
        let content = create_sample_content_with_highlighting(&highlighter);
        assert_eq!(content.path, "src/main.rs");
        assert!(!content.lines.is_empty());
    }
}
