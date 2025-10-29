# Scrollcast

A Rust CLI tool and library for converting Git repositories into formatted documents (PDF, EPUB, HTML, Markdown) with syntax highlighting.

## Features

- Convert repositories to multiple output formats
- Syntax highlighting using Syntect
- Git integration with `.gitignore` support
- Memory-efficient processing with chunking
- Pure Rust implementation

## Installation

### From source

```bash
git clone https://github.com/0xheartcode/scrollcast
cd scrollcast
cargo install --path .
```

### As a library

Add to your `Cargo.toml`:

```toml
[dependencies]
scrollcast = "0.1.0"
```

## Usage

### Command Line

```bash
# Convert to PDF
scrollcast /path/to/repo -o output.pdf

# Convert to HTML with different theme
scrollcast /path/to/repo -o output.html -f html -t zenburn

# Process all files (ignore .gitignore)
scrollcast /path/to/repo -o output.pdf --no-gitignore

# Exclude specific directories
scrollcast /path/to/repo -o output.pdf --ignore target --ignore node_modules
```

### Library

```rust
use scrollcast::{FileProcessor, MarkdownGenerator, OutputFormat, create_renderer, DocumentMetadata};
use std::path::Path;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let input_path = Path::new("./my-repo");
    let output_path = Path::new("./output.pdf");
    
    let mut processor = FileProcessor::new(input_path, true, Vec::new())?;
    let files = processor.discover_files().await?;
    
    let generator = MarkdownGenerator::new();
    let markdown = generator.generate_markdown(&files, input_path, false).await?;
    
    let metadata = DocumentMetadata {
        title: "Repository Export".to_string(),
        author: "Scrollcast".to_string(),
        created_at: chrono::Utc::now(),
    };
    
    let mut renderer = create_renderer(OutputFormat::Pdf, "kate".to_string())?;
    renderer.render(&markdown, output_path, &metadata).await?;
    
    Ok(())
}
```

## Command Line Options

```
Usage: scrollcast [OPTIONS] [input]

Arguments:
  [input]  Input directory (git repository or regular folder)

Options:
  -o, --output <output>                Output file path
  -f, --format <format>                Output format [default: pdf] [possible values: pdf, epub, html, markdown]
  -t, --theme <theme>                  Syntax highlighting theme [default: kate]
      --no-gitignore                   Ignore .gitignore files and process all files
      --no-toc                         Don't include table of contents
      --list-themes                    List available syntax highlighting themes
      --list-languages                 List supported programming languages
  -y, --yes                            Skip confirmation prompts
      --ignore <DIR>                   Ignore specific directories (can be used multiple times)
  -v, --verbose                        Enable verbose logging
      --chunk-size <chunk-size>        Process files in chunks [default: 20]
      --memory-limit <memory-limit>    Maximum memory usage in MB
      --max-file-size <max-file-size>  Maximum file size to process in MB [default: 50]
  -h, --help                           Print help
  -V, --version                        Print version
```

## Output Formats

- **PDF**: Vector graphics using `printpdf`
- **EPUB**: Reflowable documents using `epub-builder`
- **HTML**: Standalone files with embedded CSS
- **Markdown**: Clean markdown with syntax highlighting

## Syntax Highlighting

Scrollcast uses Syntect for syntax highlighting with support for common programming languages including Rust, JavaScript, Python, Go, Java, C/C++, and many others.

Available themes:
- kate (default)
- pygments
- zenburn
- breezedark
- espresso
- monochrome
- haddock
- tango

Use `--list-themes` and `--list-languages` to see all available options.

## File Processing

### Automatic Exclusions

The tool automatically excludes:
- Version control directories (`.git`, `.svn`)
- Build outputs (`target`, `dist`, `build`)
- Dependencies (`node_modules`, `vendor`)
- IDE files (`.vscode`, `.idea`)
- Binary files and archives

### Git Integration

- Respects `.gitignore` by default
- Use `--no-gitignore` to process all files
- Automatically detects Git repositories

## Performance

For large repositories, Scrollcast provides several options:
- `--chunk-size`: Process files in smaller batches
- `--memory-limit`: Limit memory usage
- `--max-file-size`: Skip very large files

## Dependencies

The library uses these main dependencies:
- `syntect` - Syntax highlighting
- `printpdf` - PDF generation
- `epub-builder` - EPUB creation
- `pulldown-cmark` - Markdown processing
- `git2` - Git integration

## License

MIT

## Contributing

Contributions are welcome. Please ensure code follows Rust conventions and includes appropriate tests.
🦀 0xheartcode
