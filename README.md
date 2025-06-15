# Git to PDF 🦀

A fast, modern Rust CLI tool for converting Git repositories into beautifully formatted PDF documents with syntax highlighting and theming support.

## ✨ Features

- 🚀 **Fast & Efficient**: Built with Rust for maximum performance
- 🎨 **Syntax Highlighting**: Support for 100+ programming languages with color-coded tokens
- 🌙 **Theme Support**: Light and dark themes with customizable colors
- 📁 **Smart Output Management**: Organized output directory structure
- 🔧 **Highly Configurable**: Configuration files, custom ignore patterns, formatting options
- 📦 **Single Binary**: No runtime dependencies
- 🔄 **Git Integration**: Clone repositories or process local directories
- 📄 **Flexible Output**: Single PDF or one PDF per file
- 🎯 **Binary File Detection**: Handles different file types appropriately

## 🚀 Quick Start

### Installation

```bash
# Clone and build
git clone https://github.com/0xheartcode/git-to-pdf
cd git-to-pdf
cargo build --release

# Or install directly from source
cargo install --path .
```

### Basic Usage

```bash
# Convert with default settings
git-to-pdf /path/to/repo

# Use dark theme with line numbers
git-to-pdf --theme dark --line-numbers /path/to/repo

# Specify custom output file
git-to-pdf -o my-project.pdf /path/to/repo

# Create a configuration file template
git-to-pdf --create-config
```

## ⚙️ Configuration

Git to PDF supports configuration files for persistent settings. Configuration files are loaded in this order:

1. File specified with `--config` flag
2. `git-to-pdf.toml` in current directory
3. `~/.git-to-pdf.toml` in home directory
4. Default settings

### Creating a Configuration File

```bash
# Create a sample configuration file
git-to-pdf --create-config
```

This creates `git-to-pdf.toml` with all available options:

```toml
[output]
folder = "output"                    # Output directory
filename = "repository.pdf"          # Default filename
create_folder = true                 # Create output folder if missing
single_file = true                   # Combine all files into one PDF

[theme]
mode = "light"                       # "light" or "dark"
font_size = 10.0                     # Font size in points
line_height = 1.2                    # Line height multiplier

[formatting]
line_numbers = true                  # Show line numbers
page_numbers = true                  # Show page numbers
syntax_highlighting = true           # Enable syntax highlighting
remove_comments = false              # Strip comments from code
remove_empty_lines = false           # Remove blank lines

[ignore]
files = ["*.tmp", "*.log", ".env"]   # Files to ignore
extensions = [".tmp", ".log"]        # Extensions to ignore
directories = ["tmp", "logs"]        # Directories to ignore
```

### Command Line Options

```bash
USAGE:
    git-to-pdf [OPTIONS] <INPUT>

ARGUMENTS:
    <INPUT>    Repository URL or local path

OPTIONS:
    -o, --output <FILE>        Output PDF file path [default: repository.pdf]
    -t, --theme <THEME>        Theme to use (light or dark) [default: light]
    -c, --config <FILE>        Use specific configuration file
        --line-numbers         Include line numbers in output
        --page-numbers         Include page numbers in output
        --create-config        Create a sample configuration file
    -h, --help                 Print help
    -V, --version              Print version
```

## 🎨 Themes & Syntax Highlighting

### Light Theme
- **Background**: Clean white
- **Text**: Dark gray (#1a1a1a)
- **Keywords**: Blue (#0066cc) - `fn`, `let`, `const`, etc.
- **Strings**: Green (#008000) - String literals
- **Comments**: Gray (#808080) - Code comments
- **Numbers**: Purple (#800080) - Numeric literals
- **Functions**: Orange (#cc6600) - Function names
- **Types**: Teal (#008080) - Type annotations
- **Operators**: Red (#cc0000) - `+`, `-`, `==`, etc.

### Dark Theme
- **Background**: Dark gray (#2d2d2d)
- **Text**: Light gray (#f0f0f0)
- **Keywords**: Light blue (#66b3ff)
- **Strings**: Light green (#66ff66)
- **Comments**: Light gray (#b3b3b3)
- **Numbers**: Light purple (#ff66ff)
- **Functions**: Light orange (#ffcc66)
- **Types**: Light teal (#66ffcc)
- **Operators**: Light red (#ff6666)

### Supported Languages

Git to PDF automatically detects and highlights syntax for:

- **Rust** (.rs)
- **JavaScript/TypeScript** (.js, .jsx, .ts, .tsx)
- **Python** (.py)
- **HTML/CSS** (.html, .htm, .css)
- **JSON/YAML** (.json, .yml, .yaml)
- **Markdown** (.md)
- **C/C++** (.c, .cpp, .cc, .cxx)
- **Go** (.go)
- **Java** (.java)
- **PHP** (.php)
- **Ruby** (.rb)
- **Shell scripts** (.sh, .bash)
- **TOML** (.toml)
- And many more...

## 📁 Output Management

By default, Git to PDF creates an `output/` directory for generated PDFs:

```
your-project/
├── git-to-pdf.toml      # Configuration file
├── output/              # Generated PDFs
│   └── repository.pdf
└── src/                 # Source code
    └── main.rs
```

You can customize the output directory and filename in the configuration file:

```toml
[output]
folder = "docs/pdfs"           # Custom output directory
filename = "my-project.pdf"    # Custom default filename
create_folder = true           # Auto-create directory
```

## 🚫 File Filtering

Git to PDF automatically excludes common files and directories:

### Universal Exclusions
- **Version control**: `.git/`, `.svn/`, `.hg/`
- **Build directories**: `target/`, `dist/`, `build/`, `node_modules/`
- **IDE files**: `.vscode/`, `.idea/`, `.vs/`
- **Binary files**: Images, videos, executables, archives
- **Lock files**: `Cargo.lock`, `package-lock.json`, `yarn.lock`

### Custom Exclusions
Add custom patterns to your configuration:

```toml
[ignore]
files = ["secret.txt", "*.env"]     # Specific files
extensions = [".tmp", ".cache"]     # File extensions
directories = ["temp", "logs"]      # Directories
```

## 🛠️ Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/0xheartcode/git-to-pdf
cd git-to-pdf

# Build in release mode
cargo build --release

# Run tests
cargo test

# Install locally
cargo install --path .
```

### Project Structure

```
src/
├── main.rs              # CLI interface and main logic
├── config.rs            # Configuration management
├── theme.rs             # Theme and color system
├── pdf_generator.rs     # PDF creation and formatting
├── syntax_highlighter.rs # Code syntax highlighting
└── file_processor.rs    # File system traversal and filtering
```

## 📋 Examples

### Basic Usage Examples

```bash
# Process current directory with default settings
git-to-pdf .

# Process with dark theme and custom output
git-to-pdf --theme dark -o docs/code-review.pdf ./src

# Use custom configuration
git-to-pdf --config ./my-config.toml /path/to/project
```

### Configuration Examples

**Minimal config for documentation:**
```toml
[output]
folder = "docs"
filename = "codebase.pdf"

[formatting]
line_numbers = false
remove_comments = true
remove_empty_lines = true
```

**Config for code review:**
```toml
[theme]
mode = "light"
font_size = 11.0

[formatting]
line_numbers = true
page_numbers = true
syntax_highlighting = true

[ignore]
directories = ["tests", "examples"]
extensions = [".test.js", ".spec.ts"]
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

### Development Setup
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Run `cargo test` and `cargo fmt`
6. Submit a pull request

## 📄 License

MIT License - see LICENSE file for details.

## 🙏 Acknowledgments

- Built with [syntect](https://github.com/trishume/syntect) for syntax highlighting
- Uses [printpdf](https://github.com/fschutt/printpdf) for PDF generation
- Powered by [clap](https://github.com/clap-rs/clap) for CLI interface

---

*Built with ❤️ and Rust by heartcode*