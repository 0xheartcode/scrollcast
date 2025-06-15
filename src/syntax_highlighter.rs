use anyhow::{Context, Result};
use std::collections::HashMap;
use syntect::highlighting::{Color, FontStyle, Style, Theme, ThemeSet};
use syntect::parsing::{SyntaxSet, SyntaxReference};
use syntect::util::as_24_bit_terminal_escaped;
use syntect::{html, parsing, highlighting};

use crate::pdf_generator::{CodeLine, HighlightedToken};
use crate::theme::Theme as AppTheme;

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    app_theme: AppTheme,
}

impl SyntaxHighlighter {
    pub fn new(app_theme: AppTheme) -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        
        Self {
            syntax_set,
            theme_set,
            app_theme,
        }
    }

    pub fn highlight_code(&self, code: &str, file_extension: &str) -> Result<Vec<CodeLine>> {
        let syntax = self.detect_syntax(file_extension);
        
        let lines: Vec<&str> = code.lines().collect();
        let mut highlighted_lines = Vec::new();

        // Use a simple approach for now - we'll map token types to our custom colors
        let mut parsing_state = parsing::ParseState::new(syntax);
        
        for line in lines {
            let ops = parsing_state.parse_line(line, &self.syntax_set)
                .context("Failed to parse line")?;
            
            let highlighted_tokens = self.process_line_ops(&ops, line);
            
            let code_line = CodeLine {
                content: line.to_string(),
                highlighted_content: Some(highlighted_tokens),
            };
            
            highlighted_lines.push(code_line);
        }

        Ok(highlighted_lines)
    }

    fn detect_syntax(&self, file_extension: &str) -> &SyntaxReference {
        // Map file extensions to syntax
        let extension = file_extension.trim_start_matches('.');
        
        self.syntax_set.find_syntax_by_extension(extension)
            .or_else(|| {
                // Try by file name patterns
                match extension {
                    "rs" => self.syntax_set.find_syntax_by_name("Rust"),
                    "py" => self.syntax_set.find_syntax_by_name("Python"),
                    "js" | "jsx" => self.syntax_set.find_syntax_by_name("JavaScript"),
                    "ts" | "tsx" => self.syntax_set.find_syntax_by_name("TypeScript"),
                    "html" | "htm" => self.syntax_set.find_syntax_by_name("HTML"),
                    "css" => self.syntax_set.find_syntax_by_name("CSS"),
                    "json" => self.syntax_set.find_syntax_by_name("JSON"),
                    "yml" | "yaml" => self.syntax_set.find_syntax_by_name("YAML"),
                    "toml" => self.syntax_set.find_syntax_by_name("TOML"),
                    "md" => self.syntax_set.find_syntax_by_name("Markdown"),
                    "sh" | "bash" => self.syntax_set.find_syntax_by_name("Bash"),
                    "c" => self.syntax_set.find_syntax_by_name("C"),
                    "cpp" | "cc" | "cxx" => self.syntax_set.find_syntax_by_name("C++"),
                    "go" => self.syntax_set.find_syntax_by_name("Go"),
                    "java" => self.syntax_set.find_syntax_by_name("Java"),
                    "php" => self.syntax_set.find_syntax_by_name("PHP"),
                    "rb" => self.syntax_set.find_syntax_by_name("Ruby"),
                    _ => None,
                }
            })
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text())
    }

    fn process_line_ops(&self, ops: &[(usize, parsing::ScopeStackOp)], line: &str) -> Vec<HighlightedToken> {
        let mut tokens = Vec::new();
        let mut current_pos = 0;
        
        for &(pos, ref op) in ops {
            if pos > current_pos {
                // Add text before this operation
                let text = &line[current_pos..pos];
                if !text.is_empty() {
                    tokens.push(HighlightedToken {
                        content: text.to_string(),
                        token_type: "text".to_string(),
                    });
                }
                current_pos = pos;
            }
            
            // Determine token type based on scope
            let token_type = match op {
                parsing::ScopeStackOp::Push(scope) => {
                    self.scope_to_token_type(scope.to_string())
                }
                _ => "text".to_string(),
            };

            // For simplicity, we'll create tokens at scope boundaries
            // In a full implementation, we'd track the scope stack properly
            if pos < line.len() {
                let remaining = &line[pos..];
                if let Some(next_space) = remaining.find(|c: char| c.is_whitespace()) {
                    let word = &remaining[..next_space];
                    if !word.is_empty() {
                        tokens.push(HighlightedToken {
                            content: word.to_string(),
                            token_type,
                        });
                        current_pos = pos + next_space;
                    }
                }
            }
        }
        
        // Add any remaining text
        if current_pos < line.len() {
            let remaining = &line[current_pos..];
            if !remaining.is_empty() {
                tokens.push(HighlightedToken {
                    content: remaining.to_string(),
                    token_type: "text".to_string(),
                });
            }
        }

        // If no tokens were created, treat the whole line as text
        if tokens.is_empty() && !line.is_empty() {
            tokens.push(HighlightedToken {
                content: line.to_string(),
                token_type: "text".to_string(),
            });
        }

        tokens
    }

    fn scope_to_token_type(&self, scope: String) -> String {
        let scope_lower = scope.to_lowercase();
        
        if scope_lower.contains("keyword") {
            "keyword".to_string()
        } else if scope_lower.contains("string") {
            "string".to_string()
        } else if scope_lower.contains("comment") {
            "comment".to_string()
        } else if scope_lower.contains("constant.numeric") {
            "number".to_string()
        } else if scope_lower.contains("entity.name.function") {
            "function".to_string()
        } else if scope_lower.contains("entity.name.type") || scope_lower.contains("storage.type") {
            "type".to_string()
        } else if scope_lower.contains("keyword.operator") {
            "operator".to_string()
        } else {
            "text".to_string()
        }
    }

    /// Simple syntax highlighting for common patterns when syntect fails
    pub fn highlight_simple(&self, code: &str, _file_extension: &str) -> Vec<CodeLine> {
        let lines: Vec<&str> = code.lines().collect();
        let mut highlighted_lines = Vec::new();

        for line in lines {
            let tokens = self.simple_tokenize(line);
            let code_line = CodeLine {
                content: line.to_string(),
                highlighted_content: Some(tokens),
            };
            highlighted_lines.push(code_line);
        }

        highlighted_lines
    }

    fn simple_tokenize(&self, line: &str) -> Vec<HighlightedToken> {
        let mut tokens = Vec::new();
        let trimmed = line.trim_start();
        
        // Handle comments
        if trimmed.starts_with("//") || trimmed.starts_with("#") || trimmed.starts_with("/*") {
            tokens.push(HighlightedToken {
                content: line.to_string(),
                token_type: "comment".to_string(),
            });
            return tokens;
        }

        // Simple word-based tokenization
        let words: Vec<&str> = line.split_whitespace().collect();
        let mut current_pos = 0;
        
        for word in words {
            // Find the position of this word in the original line
            if let Some(word_pos) = line[current_pos..].find(word) {
                let actual_pos = current_pos + word_pos;
                
                // Add whitespace before the word if any
                if actual_pos > current_pos {
                    tokens.push(HighlightedToken {
                        content: line[current_pos..actual_pos].to_string(),
                        token_type: "text".to_string(),
                    });
                }
                
                // Determine token type
                let token_type = self.classify_word(word);
                tokens.push(HighlightedToken {
                    content: word.to_string(),
                    token_type,
                });
                
                current_pos = actual_pos + word.len();
            }
        }
        
        // Add any remaining characters
        if current_pos < line.len() {
            tokens.push(HighlightedToken {
                content: line[current_pos..].to_string(),
                token_type: "text".to_string(),
            });
        }

        tokens
    }

    fn classify_word(&self, word: &str) -> String {
        // Common keywords across languages
        let keywords = [
            "fn", "let", "mut", "const", "if", "else", "for", "while", "loop", "match",
            "struct", "enum", "impl", "trait", "use", "mod", "pub", "crate", "self", "super",
            "return", "break", "continue", "async", "await", "unsafe", "extern",
            "function", "var", "class", "interface", "extends", "implements", "import", "export",
            "def", "lambda", "try", "except", "finally", "with", "as", "from",
            "int", "String", "bool", "f32", "f64", "i32", "i64", "usize", "Vec", "HashMap",
        ];

        // Check for string literals
        if (word.starts_with('"') && word.ends_with('"')) || 
           (word.starts_with('\'') && word.ends_with('\'')) {
            return "string".to_string();
        }

        // Check for numbers
        if word.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '_') {
            return "number".to_string();
        }

        // Check for keywords
        if keywords.contains(&word) {
            return "keyword".to_string();
        }

        // Check for function calls (word followed by '(')
        if word.ends_with('(') {
            return "function".to_string();
        }

        // Check for operators
        if ["==", "!=", "<=", ">=", "&&", "||", "->", "=>", "::"].contains(&word) {
            return "operator".to_string();
        }

        "text".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn test_syntax_detection() {
        let theme = Theme::light();
        let highlighter = SyntaxHighlighter::new(theme);
        
        let rust_syntax = highlighter.detect_syntax("rs");
        assert!(rust_syntax.name.contains("Rust") || rust_syntax.name == "Plain Text");
    }

    #[test]
    fn test_simple_highlighting() {
        let theme = Theme::light();
        let highlighter = SyntaxHighlighter::new(theme);
        
        let code = "fn main() {\n    let x = 42;\n    println!(\"Hello\");\n}";
        let highlighted = highlighter.highlight_simple(code, "rs");
        
        assert_eq!(highlighted.len(), 4);
        assert!(highlighted[0].highlighted_content.is_some());
    }

    #[test]
    fn test_word_classification() {
        let theme = Theme::light();
        let highlighter = SyntaxHighlighter::new(theme);
        
        assert_eq!(highlighter.classify_word("fn"), "keyword");
        assert_eq!(highlighter.classify_word("\"hello\""), "string");
        assert_eq!(highlighter.classify_word("42"), "number");
        assert_eq!(highlighter.classify_word("println!"), "text");
    }
}