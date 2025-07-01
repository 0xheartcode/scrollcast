use anyhow::{Context, Result};
use clap::{Arg, ArgAction, Command};
use colorful::{Colorful, Color};
use std::path::{Path, PathBuf};
use std::fs;
use tokio;
use dialoguer::Confirm;
use sysinfo::System;

mod config;
mod file_processor;
mod markdown_generator;
mod pandoc;
mod theme;

use file_processor::FileProcessor;
use markdown_generator::{FileInfo, MarkdownGenerator};
use pandoc::{PandocConfig, PandocConverter, OutputFormat};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("scrollcast")
        .version("0.1.0")
        .author("heartcode <0xheartcode@gmail.com>")
        .about("Convert Git repositories to beautifully formatted documents")
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
        .arg(
            Arg::new("yes")
                .short('y')
                .long("yes")
                .help("Skip confirmation prompts")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("ignore")
                .long("ignore")
                .help("Ignore specific directories (can be used multiple times)")
                .action(ArgAction::Append)
                .value_name("DIR")
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .help("Enable verbose logging")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("chunk-size")
                .long("chunk-size")
                .help("Process files in chunks of this size to reduce memory usage")
                .value_parser(clap::value_parser!(usize))
                .default_value("20")
        )
        .arg(
            Arg::new("memory-limit")
                .long("memory-limit")
                .help("Maximum memory usage in MB (default: 80% of available RAM)")
                .value_parser(clap::value_parser!(u64))
        )
        .arg(
            Arg::new("max-file-size")
                .long("max-file-size")
                .help("Maximum file size to process in MB (default: 50MB, files larger will be truncated)")
                .value_parser(clap::value_parser!(u64))
                .default_value("50")
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
    let skip_confirmation = matches.get_flag("yes");
    let verbose = matches.get_flag("verbose");
    let chunk_size = *matches.get_one::<usize>("chunk-size").unwrap();
    let memory_limit_mb = matches.get_one::<u64>("memory-limit").copied();
    let max_file_size_mb = *matches.get_one::<u64>("max-file-size").unwrap();
    let ignored_dirs: Vec<String> = matches
        .get_many::<String>("ignore")
        .unwrap_or_default()
        .map(|s| s.to_string())
        .collect();

    // Initialize system info for memory monitoring
    let mut sys = System::new();
    sys.refresh_memory();
    
    let total_memory_mb = sys.total_memory() / 1024 / 1024;
    let memory_limit = memory_limit_mb.unwrap_or(total_memory_mb * 80 / 100); // 80% of total RAM by default

    // Parse output format
    let output_format = match format.as_str() {
        "pdf" => OutputFormat::Pdf,
        "epub" => OutputFormat::Epub,
        "html" => OutputFormat::Html,
        "markdown" => OutputFormat::Markdown,
        _ => unreachable!(), // clap ensures this won't happen
    };

    // Print startup information
    println!("{}", "🎨 Scrollcast Document Converter".color(Color::Blue).bold());
    println!("📂 Input: {}", input_path.display());
    println!("📄 Output: {}", output_path.display());
    println!("🎯 Format: {}", format.clone().color(Color::Green));
    println!("🎨 Theme: {}", theme.clone().color(Color::Yellow));
    println!("📁 Respect .gitignore: {}", if respect_gitignore { "Yes".color(Color::Green) } else { "No".color(Color::Red) });
    if verbose {
        println!("🔍 Verbose mode: {}", "Enabled".color(Color::Green));
        println!("📦 Chunk size: {} files per chunk", chunk_size);
        println!("🧠 Memory limit: {} MB ({} MB total)", memory_limit, total_memory_mb);
    }

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
        .with_gitignore_respect(respect_gitignore)
        .with_ignored_directories(ignored_dirs);

    let files = file_processor.process_directory(input_path)
        .context("Failed to process input directory")?;

    if files.is_empty() {
        println!("{}", "⚠️  No files found to process".color(Color::Yellow));
        return Ok(());
    }

    println!("✅ Found {} files to process", files.len());
    
    if verbose {
        println!("📋 Files to process:");
        for (i, file) in files.iter().enumerate() {
            println!("   {}. {} ({})", i + 1, file.path, format_file_size(file.size));
        }
    }

    // Determine intelligent chunk sizing
    let total_size: usize = files.iter().map(|f| f.size).sum();
    let avg_file_size = if files.len() > 0 { total_size / files.len() } else { 0 };
    let large_files = files.iter().filter(|f| f.size > 50_000).count(); // Files > 50KB
    let huge_files = files.iter().filter(|f| f.size > 10_000_000).count(); // Files > 10MB
    let max_file_size = files.iter().map(|f| f.size).max().unwrap_or(0);
    
    // Check for extremely large files that need special handling
    if huge_files > 0 {
        println!("⚠️  Warning: Found {} files larger than 10MB. Largest file: {}", 
            huge_files, format_file_size(max_file_size));
        if max_file_size > 50_000_000 { // 50MB+
            println!("🚨 Files over 50MB may cause memory issues. Consider using --ignore to exclude them.");
        }
    }
    
    // Intelligent chunking logic
    let effective_chunk_size = if files.len() > 10 || total_size > 1_000_000 || large_files > 5 {
        // Use file-by-file processing for large repos or large files
        1
    } else if files.len() > 5 || avg_file_size > 10_000 {
        // Use small chunks for medium repos
        3
    } else {
        // Use default chunk size for small repos
        chunk_size
    };
    
    let needs_chunking = files.len() > effective_chunk_size;
    if needs_chunking {
        if effective_chunk_size == 1 {
            println!("📄 Processing {} files one-by-one for optimal memory usage", files.len());
        } else {
            println!("📦 Processing {} files in chunks of {} to reduce memory usage", files.len(), effective_chunk_size);
        }
        
        if verbose {
            println!("📊 Repository stats: {} files, {} total, avg {} per file, {} large files (>50KB)", 
                files.len(), format_file_size(total_size), format_file_size(avg_file_size), large_files);
            if huge_files > 0 {
                println!("📊 Large file stats: {} files >10MB, largest: {}", 
                    huge_files, format_file_size(max_file_size));
            }
        }
    }

    // Ask for confirmation unless -y flag is used
    if !skip_confirmation {
        let proceed = Confirm::new()
            .with_prompt("Do you want to proceed with processing these files?")
            .default(true)
            .interact()
            .context("Failed to get user confirmation")?;
        
        if !proceed {
            println!("Operation cancelled by user.");
            return Ok(());
        }
    }

    // Generate markdown
    println!("{}", "📝 Generating markdown...".color(Color::Cyan));
    let repo_name = input_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Repository");

    let temp_dir = std::env::temp_dir();
    let temp_markdown = temp_dir.join(format!("{}_temp.md", repo_name));

    if needs_chunking {
        process_files_in_chunks(files, repo_name, effective_chunk_size, &temp_markdown, include_toc, verbose, memory_limit, max_file_size_mb).await
            .context("Failed to process files in chunks")?;
    } else {
        let markdown_generator = MarkdownGenerator::new(include_toc, true);
        let markdown_content = markdown_generator.generate_markdown(files, repo_name)
            .context("Failed to generate markdown")?;
        fs::write(&temp_markdown, &markdown_content)
            .context("Failed to write temporary markdown file")?;
    }

    println!("✅ Markdown generated");
    
    if verbose {
        let markdown_size = fs::metadata(&temp_markdown)?.len();
        println!("📄 Markdown file size: {} bytes", markdown_size);
        println!("📂 Temporary markdown file: {}", temp_markdown.display());
    }

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
    converter.convert_markdown_to_document(&temp_markdown, output_path, verbose).await
        .context("Failed to convert markdown to final format")?;

    // Keep temporary file for debugging
    // let _ = fs::remove_file(&temp_markdown);
    println!("📝 Debug: Temporary markdown file: {}", temp_markdown.display());

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

async fn process_files_in_chunks(
    files: Vec<FileInfo>,
    repo_name: &str,
    chunk_size: usize,
    output_path: &Path,
    include_toc: bool,
    verbose: bool,
    memory_limit_mb: u64,
    max_file_size_mb: u64,
) -> Result<()> {
    let mut sys = System::new();
    let mut final_markdown = String::new();
    
    // Add title and metadata
    final_markdown.push_str(&format!("# {}\n\n", repo_name));
    final_markdown.push_str(&format!("Generated on: {}\n\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
    
    // Add table of contents for all files
    if include_toc {
        final_markdown.push_str("## Table of Contents\n\n");
        for file in &files {
            let sanitized_path = file.path.replace(['/', '\\'], "-").replace('.', "-");
            let escaped_path = escape_markdown_special_chars(&file.path);
            final_markdown.push_str(&format!("- [{}](#{sanitized_path})\n", escaped_path));
        }
        final_markdown.push_str("\n");
    }
    
    // Add file tree
    final_markdown.push_str("## File Structure\n\n");
    final_markdown.push_str("```\n");
    for file in &files {
        final_markdown.push_str(&format!("{}\n", file.path));
    }
    final_markdown.push_str("```\n\n");
    
    // Add file contents section header
    final_markdown.push_str("## File Contents\n\n");
    
    // Process files in chunks
    let chunks: Vec<&[FileInfo]> = files.chunks(chunk_size).collect();
    let total_chunks = chunks.len();
    let mut global_page_number = 1; // Start after title/TOC page
    let mut file_counter = 0;
    
    for (chunk_index, chunk) in chunks.iter().enumerate() {
        if verbose {
            println!("📄 Processing chunk {} of {} ({} files)", 
                chunk_index + 1, total_chunks, chunk.len());
        }
        
        // Process each file in the chunk
        for file in chunk.iter() {
            file_counter += 1;
            global_page_number += 1; // Each file gets a new page
            
            if verbose {
                sys.refresh_memory();
                let used_memory_mb = sys.used_memory() / 1024 / 1024;
                let file_size_str = if file.size > 10_000_000 {
                    format!("{} ⚠️", format_file_size(file.size))
                } else {
                    format_file_size(file.size)
                };
                println!("   📄 Processing file {}/{}: {} ({}) [Memory: {} MB/{} MB]", 
                    file_counter, files.len(), file.path, file_size_str, 
                    used_memory_mb, memory_limit_mb);
                
                if used_memory_mb > memory_limit_mb {
                    println!("⚠️  Warning: Memory usage ({} MB) exceeds limit ({} MB)", 
                        used_memory_mb, memory_limit_mb);
                }
            }
            
            // Handle very large files to prevent memory issues
            let max_file_size_bytes = max_file_size_mb * 1024 * 1024;
            let processed_content = if file.size > max_file_size_bytes as usize {
                if verbose {
                    println!("   🔄 File too large ({}), showing first 100KB + summary", format_file_size(file.size));
                }
                truncate_large_file_content(&file.content, file.size)
            } else {
                process_content_for_latex(&file.content)
            };
            
            // Add page break before each file (except the first one)
            final_markdown.push_str("\n\\newpage\n\n");
            let sanitized_path = file.path.replace(['/', '\\'], "-").replace('.', "-");
            let escaped_path = escape_markdown_special_chars(&file.path);
            
            // Add file header with page numbers
            final_markdown.push_str(&format!("### {} {{#{sanitized_path}}}\n\n", escaped_path));
            final_markdown.push_str(&format!("**File:** {} | **Size:** {} | **File #{} | Page {}**\n\n", 
                escaped_path, format_file_size(file.size), file_counter, global_page_number));
            
            if let Some(language) = &file.language {
                final_markdown.push_str(&format!("```{}\n", language));
            } else {
                final_markdown.push_str("```\n");
            }
            
            final_markdown.push_str(&processed_content);
            
            if !file.content.ends_with('\n') {
                final_markdown.push('\n');
            }
            
            final_markdown.push_str("```\n\n");
            
            // Add file info with page numbering
            final_markdown.push_str(&format!("*File size: {} | File #{} of {} | Page {}*\n\n", 
                format_file_size(file.size), file_counter, files.len(), global_page_number));
            final_markdown.push_str("---\n\n");
        }
        
        // Optional: Force garbage collection after each chunk to free memory
        // This helps with large repositories
        if chunk_index < total_chunks - 1 {
            // Allow some time for garbage collection between chunks
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }
    
    // Write the final markdown file
    fs::write(output_path, final_markdown)
        .context("Failed to write chunked markdown file")?;
    
    Ok(())
}

fn escape_markdown_special_chars(text: &str) -> String {
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

fn format_file_size(size: usize) -> String {
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

fn process_content_for_latex(content: &str) -> String {
    // Break very long lines to prevent LaTeX "dimension too large" errors
    let lines: Vec<&str> = content.lines().collect();
    let mut processed_lines = Vec::new();
    
    for line in lines {
        if line.len() > 100 {
            // Break long lines at reasonable breakpoints
            let mut current_line = String::new();
            let chars: Vec<char> = line.chars().collect();
            
            for &ch in chars.iter() {
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

fn truncate_large_file_content(content: &str, original_size: usize) -> String {
    const PREVIEW_SIZE: usize = 100_000; // Show first 100KB
    const SAMPLE_SIZE: usize = 10_000;   // Then 10KB samples
    const MAX_SAMPLES: usize = 5;        // Max 5 samples
    
    if content.len() <= PREVIEW_SIZE {
        return process_content_for_latex(content);
    }
    
    let mut result = String::new();
    
    // Add first part
    let preview_end = std::cmp::min(PREVIEW_SIZE, content.len());
    result.push_str(&process_content_for_latex(&content[..preview_end]));
    result.push_str("\n\n");
    result.push_str(&format!("... [File continues for {} more] ...\n\n", 
        format_file_size(original_size - preview_end)));
    
    // Add samples from the middle and end
    let remaining = content.len() - preview_end;
    if remaining > SAMPLE_SIZE * 2 {
        for i in 1..=MAX_SAMPLES {
            let sample_start = preview_end + (remaining * i) / (MAX_SAMPLES + 1);
            let sample_end = std::cmp::min(sample_start + SAMPLE_SIZE, content.len());
            
            if sample_start < content.len() {
                result.push_str(&format!("\n--- Sample {} (around {}%) ---\n", 
                    i, (sample_start * 100) / content.len()));
                result.push_str(&process_content_for_latex(&content[sample_start..sample_end]));
                result.push_str("\n");
            }
        }
    }
    
    // Add summary
    result.push_str(&format!("\n\n--- File Summary ---\n"));
    result.push_str(&format!("Total size: {}\n", format_file_size(original_size)));
    result.push_str(&format!("Lines shown: ~{} of ~{}\n", 
        result.lines().count(), content.lines().count()));
    result.push_str("Note: Large file truncated to prevent memory issues.\n");
    
    result
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
