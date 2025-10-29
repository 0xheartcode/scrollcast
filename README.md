# Scrollcast 🦀📄

A modern Rust CLI tool that converts Git repositories into beautifully formatted documents (PDF, EPUB, HTML, Markdown) with professional syntax highlighting and theming. Pure Rust implementation with no external dependencies.

## ✨ Features

- 🚀 **Fast & Efficient**: Built with Rust for maximum performance
- 🦀 **Pure Rust**: No external dependencies - everything built with Rust libraries
- 📚 **Multiple Formats**: PDF, EPUB (experimental), HTML, and Markdown output
- 🌈 **Syntax Highlighting**: Support for 100+ programming languages via Syntect
- 🔧 **Smart Language Detection**: Automatic syntax detection and highlighting
- 📁 **Smart Git Integration**: Respects .gitignore by default with override options
- 🎯 **Binary File Detection**: Intelligent handling of different file types
- 📋 **Table of Contents**: Automatic TOC generation for easy navigation
- 🎨 **Themes**: Multiple syntax highlighting themes (kate, pygments, zenburn, etc.)
- ⚡ **Zero Configuration**: Works out of the box with sensible defaults

## 🚀 Quick Start

### Prerequisites

**No external dependencies required!** Scrollcast is a pure Rust implementation that works out of the box.

- Rust 1.70+ (for building from source)
- Git (for processing git repositories)

### Installation

```bash
# Clone and build
git clone https://github.com/0xheartcode/scrollcast
cd scrollcast
cargo build --release

# Run the binary
./target/release/scrollcast
```

### Basic Usage

```bash
# Convert to PDF (default)
scrollcast /path/to/repo -o output.pdf

# Convert to EPUB (experimental)
scrollcast /path/to/repo -o output.epub -f epub --include-experimental

# Convert to HTML
scrollcast /path/to/repo -o output.html -f html

# Convert to Markdown
scrollcast /path/to/repo -o output.md -f markdown

# Test project generation
scrollcast --test-project

# Use different theme
scrollcast /path/to/repo -o output.pdf -t zenburn

# Ignore .gitignore files
scrollcast /path/to/repo -o output.pdf --no-gitignore

# Ignore specific directories
scrollcast /path/to/repo -o output.pdf --ignore lib --ignore node_modules

# Skip confirmation prompts
scrollcast /path/to/repo -o output.pdf -y
```

## 📖 Command Line Options

```bash
USAGE:
    scrollcast [OPTIONS] <INPUT>

ARGUMENTS:
    <INPUT>    Input directory (git repository or regular folder)

OPTIONS:
    -o, --output <FILE>        Output file path [required]
    -f, --format <FORMAT>      Output format [default: pdf] [possible values: pdf, epub, html, markdown]
    -t, --theme <THEME>        Syntax highlighting theme [default: kate]
                              [possible values: pygments, kate, monochrome, breezedark, espresso, zenburn, haddock, tango]
        --no-gitignore         Ignore .gitignore files and process all files
        --no-toc               Don't include table of contents
        --ignore <DIR>         Ignore specific directories (can be used multiple times)
        --test-project         Generate test project documentation in testfiles/output_test
        --include-experimental Include experimental features like EPUB
    -y, --yes                  Skip confirmation prompts
        --list-themes          List available syntax highlighting themes
        --list-languages       List supported programming languages
    -h, --help                 Print help
    -V, --version              Print version
```

## 🎨 Output Formats

### PDF
- **Engine**: printpdf for pure Rust implementation
- **Features**: Vector graphics, professional typography
- **Best for**: Printing, sharing, archival

### EPUB (Experimental)
- **Features**: Reflowable text, TOC navigation, inline CSS styling
- **Best for**: E-readers, mobile devices, accessibility
- **Note**: Include with `--include-experimental` flag

### HTML
- **Features**: Standalone HTML with embedded CSS, responsive design
- **Best for**: Web publishing, online documentation

### Markdown
- **Features**: Clean markdown with syntax highlighting metadata
- **Best for**: GitHub wikis, documentation platforms

## 🌈 Syntax Highlighting

Scrollcast automatically detects and highlights syntax for 300+ programming languages including:

**Popular Languages:**
- Rust, JavaScript/TypeScript, Python, Go, Java, C/C++
- HTML/CSS, JSON/YAML, Markdown, Shell scripts
- PHP, Ruby, Swift, Kotlin, Scala, Haskell
- SQL, Docker, YAML, TOML, XML

**Blockchain & Smart Contracts:**
- **Solidity** (using JavaScript syntax highlighting)
- Vyper, Move, Cairo

**Web Technologies:**
- React JSX, Vue, Angular templates
- SASS/SCSS, Less, Stylus
- GraphQL, WASM

**Data & Config:**
- JSON, YAML, TOML, XML, CSV
- Dockerfile, Kubernetes YAML
- Terraform, Ansible

Use `--list-languages` to see all supported languages.

## 🎨 Available Themes

**Light Background Themes:**
- **kate** (default) - Balanced colors, excellent readability
- **pygments** - Classic Python documentation style
- **tango** - GNOME's colorful theme
- **haddock** - Haskell documentation style
- **monochrome** - Black and white for printing

**Dark Background Themes:**
- **zenburn** - Dark, low-contrast theme
- **breezedark** - KDE's dark theme
- **espresso** - Rich coffee-inspired colors

Use `--list-themes` to see all available themes.

## 📁 File Processing

### Universal Exclusions

Scrollcast automatically excludes:

**Directories:**
- Version control: `.git`, `.svn`, `.hg`
- Build outputs: `target`, `dist`, `build`, `out`
- Dependencies: `node_modules`, `vendor`, `.cargo`
- IDE files: `.vscode`, `.idea`, `.vs`
- Cache: `.cache`, `__pycache__`, `.pytest_cache`

**Files:**
- Lock files: `Cargo.lock`, `package-lock.json`, `yarn.lock`
- IDE configs: `.editorconfig`
- System files: `.DS_Store`, `Thumbs.db`

**Extensions:**
- Media: `.png`, `.jpg`, `.mp4`, `.mp3`, etc.
- Archives: `.zip`, `.tar`, `.gz`, `.rar`, etc.
- Executables: `.exe`, `.dll`, `.so`, `.bin`, etc.
- Documents: `.pdf`, `.doc`, `.xls`, `.ps`, etc.

### Git Integration

- **Respects .gitignore**: By default, follows your repository's ignore rules
- **Override option**: Use `--no-gitignore` to process all files
- **Smart detection**: Automatically detects git repositories

## 🔧 Advanced Usage

### Custom Output Structure

The tool generates a structured document with:

1. **Title page** with repository name and generation timestamp
2. **Table of contents** (unless `--no-toc` is specified)
3. **File structure** tree view of processed files
4. **File contents** with syntax highlighting and metadata

### Processing Large Repositories

For large repositories:

```bash
# Use .gitignore to limit scope
scrollcast large-repo/ -o output.pdf

# Or create a custom .gitignore in the repo root
echo "docs/" >> .gitignore
echo "tests/" >> .gitignore
scrollcast large-repo/ -o output.pdf
```

### Solidity Development

Solidity files are highlighted using JavaScript syntax highlighting for optimal compatibility:

```bash
# Process smart contracts with syntax highlighting
scrollcast my-smart-contracts/ -o contracts.pdf

# Generate EPUB documentation
scrollcast my-dapp/ -o dapp-code.epub -f epub
```

## 🛠️ Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/0xheartcode/scrollcast
cd scrollcast

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
├── main.rs                 # CLI interface and orchestration
├── config.rs              # Configuration management
├── file_processor.rs      # File discovery and filtering
├── markdown_generator.rs  # Markdown generation from file tree
├── theme.rs               # Theme definitions
└── renderer/
    ├── mod.rs             # Document renderer trait and core logic
    ├── html.rs            # HTML renderer with syntax highlighting
    ├── epub.rs            # EPUB renderer with inline CSS
    └── pdf.rs             # PDF renderer (printpdf-based)
```

### Dependencies

- **pulldown-cmark**: Markdown parsing and processing
- **syntect**: Syntax highlighting engine
- **printpdf**: PDF generation
- **epub-builder**: EPUB document creation
- **clap**: Command-line argument parsing
- **git2**: Git repository integration
- **regex**: Pattern matching for CSS conversion
- **tokio**: Async runtime

## 📋 Examples

### Documentation Generation

```bash
# Generate project documentation
scrollcast ./my-project -o docs/codebase.pdf -t kate

# Create EPUB for mobile reading (experimental)
scrollcast ./my-project -o docs/codebase.epub -f epub -t breezedark --include-experimental
```

### Code Review

```bash
# Generate HTML for web-based review
scrollcast ./feature-branch -o review.html -f html --no-toc

# PDF with monochrome theme for printing
scrollcast ./feature-branch -o review.pdf -t monochrome
```

### Smart Contract Auditing

```bash
# Process Solidity contracts
scrollcast ./contracts -o audit.pdf -t zenburn

# Include all files, even ignored ones
scrollcast ./full-project --no-gitignore -o complete-audit.pdf
```

## 🤝 Contributing

**We welcome contributions!** Whether you're fixing bugs, adding features, improving documentation, or sharing ideas - every contribution helps make Scrollcast better.

### Quick Start for Contributors
- 🐛 **Found a bug?** Open an issue with details
- 💡 **Have an idea?** Share it in discussions or issues  
- 🔧 **Want to code?** Check out our [Contributing Guide](CONTRIBUTING.md)
- 📚 **Improve docs?** README updates are always welcome

### What We Need Help With
- **New output formats** (Word, LaTeX, etc.)
- **Better syntax highlighting** for more languages
- **Performance improvements** for large repositories  
- **Documentation and examples**

See our [Contributing Guide](CONTRIBUTING.md) for detailed setup instructions and development workflow.

## 📄 License

MIT License

## 🙏 Acknowledgments

- **pulldown-cmark**: Fast CommonMark parser
- **syntect**: Pure Rust syntax highlighting
- **printpdf**: Rust PDF generation library
- **epub-builder**: EPUB creation toolkit
- **KDE Syntax Highlighting**: Comprehensive language definitions

---

*Built with ❤️ and Rust by [heartcode](https://github.com/0xheartcode)*