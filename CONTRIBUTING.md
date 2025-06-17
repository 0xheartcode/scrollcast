# Contributing to Scrollcast

Thank you for your interest in contributing to Scrollcast! We welcome contributions from the community.

## 🚀 Getting Started

### Prerequisites
- [Rust](https://rustup.rs/) (latest stable version)
- [Pandoc](https://pandoc.org/installing.html)
- Git

### Development Setup
1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/yourusername/scrollcast
   cd scrollcast
   ```
3. Install dependencies:
   ```bash
   cargo build
   ```
4. Run tests:
   ```bash
   cargo test
   ```

## 🛠️ Development Workflow

1. **Create a feature branch:**
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes:**
   - Write clean, well-documented code
   - Follow Rust naming conventions
   - Add tests for new functionality

3. **Test your changes:**
   ```bash
   cargo test
   cargo fmt
   cargo clippy
   ```

4. **Commit your changes:**
   ```bash
   git commit -m "Add feature: description of your changes"
   ```

5. **Push and create a pull request:**
   ```bash
   git push origin feature/your-feature-name
   ```

## 📝 What We're Looking For

### High Priority
- **New output formats** (Word, LaTeX, etc.)
- **Better syntax highlighting** for more languages
- **Performance improvements** for large repositories
- **Error handling** improvements
- **Documentation** enhancements

### Medium Priority
- **Configuration file** support
- **Custom templates** for output formatting
- **Multi-language** README translations
- **Example projects** and tutorials

### Low Priority
- **GUI interface**
- **Plugin system**
- **Cloud integration**

## 🐛 Bug Reports

When reporting bugs, please include:
- Rust version (`rustc --version`)
- Pandoc version (`pandoc --version`)
- Operating system
- Command that caused the issue
- Expected vs actual behavior
- Sample repository (if possible)

## 💡 Feature Requests

Before requesting a feature:
1. Check existing issues to avoid duplicates
2. Explain the use case and benefits
3. Consider implementation complexity
4. Provide examples if helpful

## 🧪 Testing

- Write unit tests for new functions
- Test with different file types and repositories
- Ensure all existing tests pass
- Test on different operating systems if possible

## 📚 Documentation

- Update README.md for new features
- Add inline code comments for complex logic
- Include examples in docstrings
- Update help text for new CLI options

## 🎨 Code Style

- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Follow Rust naming conventions
- Keep functions focused and small
- Use meaningful variable names

## 🤝 Community Guidelines

- Be respectful and inclusive
- Help others learn and contribute
- Provide constructive feedback
- Follow the project's code of conduct

## 📞 Getting Help

- **GitHub Issues:** For bugs and feature requests
- **GitHub Discussions:** For questions and general discussion
- **Review existing issues** before creating new ones

Thank you for contributing to Scrollcast! 🦀📄