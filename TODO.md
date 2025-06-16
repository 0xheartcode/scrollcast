# Git-to-PDF TODO List

## 🚨 CRITICAL ISSUES (Must Fix Immediately)

### 1. **Fix Word Spacing Bug** 
- **Issue**: Enormous gaps between words making PDFs unreadable
- **Cause**: Incorrect character width calculation in `add_highlighted_text()`
- **Fix**: Proper font metrics and text positioning
- **File**: `src/pdf_generator.rs:169`
- **Priority**: CRITICAL

### 2. **Fix Syntax Highlighting Colors**
- **Issue**: All text appears in black/gray - no color highlighting
- **Cause**: Colors not being applied to PDF text properly
- **Fix**: Ensure color mapping works with printpdf library
- **File**: `src/pdf_generator.rs` - color application logic
- **Priority**: HIGH

### 3. **Fix Dark Theme Background**
- **Issue**: Dark theme shows white background instead of dark
- **Cause**: Background color not being applied to PDF pages
- **Fix**: Implement proper background rectangles in PDF
- **File**: `src/pdf_generator.rs:42-47`
- **Priority**: HIGH

### 4. **Fix Missing Content/Pagination**
- **Issue**: Page 2 of comprehensive PDF is completely blank
- **Cause**: Content being lost during page breaks or processing
- **Fix**: Debug page break logic and content flow
- **File**: `src/pdf_generator.rs` - pagination logic
- **Priority**: HIGH

## 🔧 IMPLEMENTATION TASKS

### PDF Generation Fixes

#### A. **Spacing and Layout**
- [ ] Fix character width calculation (use actual font metrics)
- [ ] Implement proper word spacing using PDF text operators
- [ ] Test with different font sizes to ensure consistent spacing
- [ ] Add proper line height handling
- [ ] Fix text positioning and alignment

#### B. **Color System**
- [ ] Debug printpdf color application methods
- [ ] Ensure RGB values are correctly applied to text
- [ ] Test color rendering for all syntax elements
- [ ] Verify color mapping between themes and PDF output
- [ ] Add fallback colors for unsupported elements

#### C. **Background and Themes**
- [ ] Implement PDF page background coloring for dark theme
- [ ] Add background rectangles that cover entire page
- [ ] Ensure text colors contrast properly with backgrounds
- [ ] Test theme switching and persistence

#### D. **Content Processing**
- [ ] Debug why content is being truncated
- [ ] Fix page break calculations
- [ ] Ensure all files are being processed completely
- [ ] Test with larger codebases
- [ ] Add proper error handling for content overflow

### Syntax Highlighting Improvements

#### E. **Token Processing**
- [ ] Improve token classification accuracy
- [ ] Add support for more programming languages
- [ ] Fix edge cases in simple tokenizer
- [ ] Add proper handling for multi-line comments
- [ ] Support for string escapes and special characters

#### F. **Color Mapping**
- [ ] Verify all syntax elements have proper colors
- [ ] Add more granular token types (variables, constants, etc.)
- [ ] Implement custom color schemes
- [ ] Add color configuration validation

### File Processing Enhancements

#### G. **Real Repository Support**
- [ ] Implement Git operations module (`src/git_ops.rs`)
- [ ] Add repository cloning functionality
- [ ] Support for processing remote repositories
- [ ] Handle authentication for private repos
- [ ] Add progress indicators for large repos

#### H. **File System Integration**
- [ ] Connect file processor to main workflow
- [ ] Implement custom ignore file processing
- [ ] Add support for .gitignore patterns
- [ ] Test with real-world project structures
- [ ] Handle binary files and large files properly

### CLI and UX Improvements

#### I. **Interactive Features**
- [ ] Add interactive prompts for configuration
- [ ] Implement progress bars for PDF generation
- [ ] Add verbose output modes
- [ ] Implement dry-run mode to preview what will be processed
- [ ] Add file size and processing time estimates

#### J. **Configuration Enhancements**
- [ ] Add configuration validation
- [ ] Support for multiple output formats
- [ ] Add template configurations for different use cases
- [ ] Implement configuration migration for version updates

### Testing and Quality

#### K. **Test Coverage**
- [ ] Add integration tests for full workflow
- [ ] Test with various repository types and sizes
- [ ] Add visual regression tests for PDF output
- [ ] Test performance with large codebases
- [ ] Add error handling tests

#### L. **Documentation**
- [ ] Add inline code documentation
- [ ] Update README with troubleshooting section
- [ ] Add examples for different programming languages
- [ ] Create configuration reference guide

### Performance and Optimization

#### M. **Performance**
- [ ] Profile memory usage with large repositories
- [ ] Optimize PDF generation for speed
- [ ] Add concurrent file processing
- [ ] Implement streaming for large files
- [ ] Add caching for repeated operations

#### N. **Error Handling**
- [ ] Add comprehensive error messages
- [ ] Implement graceful degradation for unsupported files
- [ ] Add logging and debugging capabilities
- [ ] Handle network timeouts and git errors
- [ ] Add recovery mechanisms for partial failures

## 🎯 FUTURE ENHANCEMENTS

### Advanced Features
- [ ] Multiple output formats (HTML, Markdown)
- [ ] Custom fonts and typography
- [ ] Table of contents generation
- [ ] Code statistics and metrics
- [ ] Integration with CI/CD pipelines
- [ ] Web interface for online conversion
- [ ] Plugin system for custom processors

### Language Support
- [ ] Add support for more programming languages
- [ ] Custom syntax highlighting rules
- [ ] Language-specific formatting options
- [ ] Unicode and international character support

## 📋 IMMEDIATE ACTION PLAN

### Phase 1: Critical Fixes (Current Priority)
1. **Fix word spacing** - Make PDFs readable
2. **Fix syntax highlighting colors** - Add visual differentiation  
3. **Fix dark theme background** - Proper theme implementation
4. **Fix missing content** - Ensure complete processing

### Phase 2: Core Features
5. Implement Git operations for repository cloning
6. Connect file processor to main workflow
7. Add comprehensive error handling
8. Improve test coverage

### Phase 3: Polish and Performance
9. Add interactive features and progress indicators
10. Optimize performance for large repositories
11. Add advanced configuration options
12. Complete documentation

---

**Total Tasks**: 45+ individual tasks across 14 major categories
**Estimated Effort**: ~2-3 weeks for full completion
**Current Focus**: Critical PDF generation fixes (Tasks 1-4)