use anyhow::{Context, Result};
use clap::{Arg, ArgAction, Command};
use colorful::{Colorful, Color};
use std::path::PathBuf;
use std::fs;
use tokio;

mod config;
mod file_processor;
mod markdown_generator;
mod pandoc;
mod theme;

use file_processor::FileProcessor;
use markdown_generator::MarkdownGenerator;
use pandoc::{PandocConfig, PandocConverter, OutputFormat};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("git-to-pdf")
        .version("0.1.0")
        .author("heartcode <0xheartcode@gmail.com>")
        .about("Convert Git repositories to beautifully formatted PDF/EPUB documents")
        .arg(
            Arg::new("input")
                .help("Input directory (git repository or regular folder)")
                .required_unless_present_any(["list-themes", "list-languages"])
                .index(1)
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output file path")
                .required_unless_present_any(["list-themes", "list-languages"])
                .value_parser(clap::value_parser!(PathBuf))
        )
        .arg(
            Arg::new("format")
                .short('f')
                .long("format")
                .help("Output format")
                .value_parser(["pdf", "epub", "html", "markdown"])
                .default_value("pdf")
        )
        .arg(
            Arg::new("theme")
                .short('t')
                .long("theme")
                .help("Syntax highlighting theme")
                .value_parser(["pygments", "kate", "monochrome", "breezedark", "espresso", "zenburn", "haddock", "tango"])
                .default_value("kate")
        )
        .arg(
            Arg::new("no-gitignore")
                .long("no-gitignore")
                .help("Ignore .gitignore files and process all files")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("no-toc")
                .long("no-toc")
                .help("Don't include table of contents")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("list-themes")
                .long("list-themes")
                .help("List available syntax highlighting themes")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("list-languages")
                .long("list-languages")
                .help("List supported programming languages")
                .action(ArgAction::SetTrue)
        )
        .get_matches();

    // Handle list commands
    if matches.get_flag("list-themes") {
        list_themes()?;
        return Ok(());
    }

    if matches.get_flag("list-languages") {
        list_languages()?;
        return Ok(());
    }

    // Get command line arguments
    let input_path = matches.get_one::<PathBuf>("input").unwrap();
    let output_path = matches.get_one::<PathBuf>("output").unwrap();
    let format = matches.get_one::<String>("format").unwrap();
    let theme = matches.get_one::<String>("theme").unwrap().clone();
    let respect_gitignore = !matches.get_flag("no-gitignore");
    let include_toc = !matches.get_flag("no-toc");

    // Parse output format
    let output_format = match format.as_str() {
        "pdf" => OutputFormat::Pdf,
        "epub" => OutputFormat::Epub,
        "html" => OutputFormat::Html,
        "markdown" => OutputFormat::Markdown,
        _ => unreachable!(), // clap ensures this won't happen
    };

    // Print startup information
    println!("{}", "🎨 Git to Document Converter".color(Color::Blue).bold());
    println!("📂 Input: {}", input_path.display());
    println!("📄 Output: {}", output_path.display());
    println!("🎯 Format: {}", format.clone().color(Color::Green));
    println!("🎨 Theme: {}", theme.clone().color(Color::Yellow));
    println!("📁 Respect .gitignore: {}", if respect_gitignore { "Yes".color(Color::Green) } else { "No".color(Color::Red) });

    // Validate input path
    if !input_path.exists() {
        anyhow::bail!("Input path does not exist: {}", input_path.display());
    }

    // Create output directory if it doesn't exist
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .context("Failed to create output directory")?;
    }

    // Process the repository/directory
    println!("\n{}", "📖 Processing files...".color(Color::Cyan));
    let file_processor = FileProcessor::new()
        .with_gitignore_respect(respect_gitignore);

    let files = file_processor.process_directory(input_path)
        .context("Failed to process input directory")?;

    if files.is_empty() {
        println!("{}", "⚠️  No files found to process".color(Color::Yellow));
        return Ok(());
    }

    println!("✅ Found {} files to process", files.len());

    // Generate markdown
    println!("{}", "📝 Generating markdown...".color(Color::Cyan));
    let repo_name = input_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Repository");

    let markdown_generator = MarkdownGenerator::new(include_toc, true);
    let markdown_content = markdown_generator.generate_markdown(files, repo_name)
        .context("Failed to generate markdown")?;

    // Create temporary markdown file
    let temp_dir = std::env::temp_dir();
    let temp_markdown = temp_dir.join(format!("{}_temp.md", repo_name));
    fs::write(&temp_markdown, markdown_content)
        .context("Failed to write temporary markdown file")?;

    println!("✅ Markdown generated");

    // Configure Pandoc
    let pandoc_config = PandocConfig {
        output_format,
        highlight_style: theme,
        include_toc,
        syntax_definitions: Vec::new(),
    };

    let converter = PandocConverter::new(pandoc_config)
        .context("Failed to initialize Pandoc converter")?;

    // Convert to final format
    println!("{}", "🔄 Converting to final format...".color(Color::Cyan));
    converter.convert_markdown_to_document(&temp_markdown, output_path).await
        .context("Failed to convert markdown to final format")?;

    // Clean up temporary file
    let _ = fs::remove_file(&temp_markdown);

    println!("\n{} Document generated successfully!", "🎉".color(Color::Green));
    println!("📄 Output: {}", output_path.display().to_string().color(Color::Blue));

    // Show file size
    if let Ok(metadata) = fs::metadata(output_path) {
        let size = metadata.len();
        let size_str = if size > 1_048_576 {
            format!("{:.1} MB", size as f64 / 1_048_576.0)
        } else if size > 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else {
            format!("{} bytes", size)
        };
        println!("📊 File size: {}", size_str.color(Color::Green));
    }

    Ok(())
}

fn list_themes() -> Result<()> {
    println!("{}", "Available syntax highlighting themes:".color(Color::Blue).bold());
    
    match PandocConverter::list_available_highlight_styles() {
        Ok(themes) => {
            for theme in themes {
                println!("  • {}", theme.color(Color::Green));
            }
        }
        Err(e) => {
            eprintln!("Failed to get themes: {}", e);
            println!("Default themes: pygments, kate, monochrome, breezedark, espresso, zenburn, haddock, tango");
        }
    }
    
    Ok(())
}

fn list_languages() -> Result<()> {
    println!("{}", "Supported programming languages:".color(Color::Blue).bold());
    
    match PandocConverter::list_available_languages() {
        Ok(languages) => {
            // Display languages in columns
            let mut count = 0;
            for language in languages {
                print!("{:<20}", language);
                count += 1;
                if count % 4 == 0 {
                    println!();
                }
            }
            if count % 4 != 0 {
                println!();
            }
            println!("\n{}", "Note: Solidity support is automatically added when needed.".color(Color::Yellow));
        }
        Err(e) => {
            eprintln!("Failed to get supported languages: {}", e);
        }
    }
    
    Ok(())
}
