mod theme;
mod pdf_generator;

use anyhow::Result;
use clap::{Arg, Command};
use theme::{Theme, ThemeMode};
use pdf_generator::{PdfGenerator, FileContent};

fn main() -> Result<()> {
    let matches = Command::new("git-to-pdf")
        .version("0.1.0")
        .author("heartcode <0xheartcode@gmail.com>")
        .about("Convert Git repositories to beautifully formatted PDF documents")
        .arg(
            Arg::new("input")
                .help("Repository URL or local path")
                .required(true)
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
        .get_matches();

    let _input = matches.get_one::<String>("input").unwrap();
    let output = matches.get_one::<String>("output").unwrap();
    let theme_str = matches.get_one::<String>("theme").unwrap();
    let line_numbers = matches.get_flag("line-numbers");
    let page_numbers = matches.get_flag("page-numbers");

    // Parse theme
    let theme_mode = match theme_str.as_str() {
        "dark" => ThemeMode::Dark,
        "light" => ThemeMode::Light,
        _ => {
            eprintln!("Invalid theme '{}'. Using light theme.", theme_str);
            ThemeMode::Light
        }
    };

    let theme = Theme::from_mode(theme_mode);
    println!("🎨 Using {} theme", match theme.mode {
        ThemeMode::Light => "light",
        ThemeMode::Dark => "dark",
    });

    // For now, let's create a simple test PDF with some sample code
    let sample_content = create_sample_content();
    
    let generator = PdfGenerator::new(theme, line_numbers, page_numbers);
    
    println!("📄 Generating PDF...");
    generator.create_pdf(output, vec![sample_content])?;
    
    println!("✅ PDF generated successfully: {}", output);
    
    Ok(())
}

fn create_sample_content() -> FileContent {
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

    FileContent::new("src/main.rs".to_string(), rust_code.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_content_creation() {
        let content = create_sample_content();
        assert_eq!(content.path, "src/main.rs");
        assert!(!content.lines.is_empty());
    }
}
