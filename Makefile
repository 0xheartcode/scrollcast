# Scrollcast - Development Makefile

.PHONY: help build test clean install lint fmt check release example docs all

# Default target
help: ## Show this help message
	@echo "Scrollcast - Development Commands"
	@echo "================================="
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

# Build targets
build: ## Build the project in debug mode
	cargo build

release: ## Build the project in release mode
	cargo build --release

install: ## Install the binary locally
	cargo install --path .

# Development targets
test: ## Run all tests
	cargo test

check: ## Check code without building
	cargo check

lint: ## Run clippy for linting
	cargo clippy -- -D warnings

fmt: ## Format code using rustfmt
	cargo fmt

fmt-check: ## Check if code is formatted correctly
	cargo fmt -- --check

# Quality assurance
all-checks: fmt-check lint test ## Run all quality checks

# Clean targets
clean: ## Clean build artifacts
	cargo clean
	rm -rf output/
	rm -rf target/

# Example runs
example-pdf: build ## Generate example PDF from test project
	./target/debug/scrollcast test-project/ -o output/example.pdf

example-epub: build ## Generate example EPUB from test project
	./target/debug/scrollcast test-project/ -o output/example.epub -f epub

example-html: build ## Generate example HTML from test project
	./target/debug/scrollcast test-project/ -o output/example.html -f html

example-markdown: build ## Generate example Markdown from test project
	./target/debug/scrollcast test-project/ -o output/example.md -f markdown

example-all: example-pdf example-epub example-html example-markdown ## Generate all example formats

# Self-documentation
docs-pdf: release ## Generate PDF documentation of this project
	./target/release/scrollcast . -o output/scrollcast-source.pdf

docs-epub: release ## Generate EPUB documentation of this project
	./target/release/scrollcast . -o output/scrollcast-source.epub -f epub

docs-all: docs-pdf docs-epub ## Generate all documentation formats

# Development setup
setup: ## Set up development environment
	@echo "Setting up development environment..."
	@command -v pandoc >/dev/null 2>&1 || { echo >&2 "Pandoc is required but not installed. Please install it first."; exit 1; }
	@command -v xelatex >/dev/null 2>&1 || { echo >&2 "XeLaTeX is required for PDF generation. Please install texlive-xetex."; exit 1; }
	@echo "✅ Pandoc found: $$(pandoc --version | head -n1)"
	@echo "✅ XeLaTeX found: $$(xelatex --version | head -n1)"
	@echo "✅ Development environment is ready!"

# Benchmarking and performance
bench: ## Run performance benchmarks
	cargo bench

# Release preparation
pre-release: all-checks example-all docs-all ## Prepare for release (run all checks and generate examples)
	@echo "🚀 Pre-release checks completed successfully!"
	@echo "📦 Built examples and documentation in output/"

# Utility targets
size: release ## Show binary size
	@ls -lh target/release/scrollcast | awk '{print "Binary size: " $$5}'

list-themes: build ## List available syntax highlighting themes
	./target/debug/scrollcast --list-themes

list-languages: build ## List supported programming languages
	./target/debug/scrollcast --list-languages

# Output directory management
output-dir: ## Create output directory
	mkdir -p output

# Development workflow
dev: fmt lint test ## Run development workflow (format, lint, test)

# Watch mode (requires cargo-watch)
watch: ## Watch for changes and run tests (requires: cargo install cargo-watch)
	cargo watch -x test

watch-build: ## Watch for changes and build (requires: cargo install cargo-watch)
	cargo watch -x build

# Package information
info: ## Show project information
	@echo "Project: Scrollcast"
	@echo "Version: $$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].version')"
	@echo "Rust version: $$(rustc --version)"
	@echo "Cargo version: $$(cargo --version)"
	@echo "Target directory: target/"
	@echo "Output directory: output/"

# Default target when no target is specified
all: build test ## Build and test the project