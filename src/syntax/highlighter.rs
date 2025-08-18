use syntect::parsing::{SyntaxSet, SyntaxReference};
use syntect::highlighting::{ThemeSet, Style};
use syntect::util::LinesWithEndings;
use anyhow::Result;
use std::collections::HashMap;

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_theme: String,
    language_map: HashMap<String, String>,
}

impl SyntaxHighlighter {
    pub fn new() -> Result<Self> {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        
        // Create language mapping for common extensions
        let mut language_map = HashMap::new();
        
        // Map our language names to syntect syntax names
        language_map.insert("rust".to_string(), "Rust".to_string());
        language_map.insert("python".to_string(), "Python".to_string());
        language_map.insert("javascript".to_string(), "JavaScript".to_string());
        language_map.insert("typescript".to_string(), "TypeScript".to_string());
        language_map.insert("jsx".to_string(), "JavaScript (JSX)".to_string());
        language_map.insert("tsx".to_string(), "TypeScriptReact".to_string());
        language_map.insert("html".to_string(), "HTML".to_string());
        language_map.insert("css".to_string(), "CSS".to_string());
        language_map.insert("scss".to_string(), "Sass".to_string());
        language_map.insert("json".to_string(), "JSON".to_string());
        language_map.insert("xml".to_string(), "XML".to_string());
        language_map.insert("yaml".to_string(), "YAML".to_string());
        language_map.insert("toml".to_string(), "TOML".to_string());
        language_map.insert("markdown".to_string(), "Markdown".to_string());
        language_map.insert("bash".to_string(), "Bourne Again Shell (bash)".to_string());
        language_map.insert("sh".to_string(), "Shell-Unix-Generic".to_string());
        language_map.insert("zsh".to_string(), "Shell-Unix-Generic".to_string());
        language_map.insert("fish".to_string(), "Shell-Unix-Generic".to_string());
        language_map.insert("c".to_string(), "C".to_string());
        language_map.insert("cpp".to_string(), "C++".to_string());
        language_map.insert("go".to_string(), "Go".to_string());
        language_map.insert("java".to_string(), "Java".to_string());
        language_map.insert("kotlin".to_string(), "Kotlin".to_string());
        language_map.insert("swift".to_string(), "Swift".to_string());
        language_map.insert("php".to_string(), "PHP".to_string());
        language_map.insert("ruby".to_string(), "Ruby".to_string());
        language_map.insert("perl".to_string(), "Perl".to_string());
        language_map.insert("lua".to_string(), "Lua".to_string());
        language_map.insert("r".to_string(), "R".to_string());
        language_map.insert("sql".to_string(), "SQL".to_string());
        language_map.insert("dockerfile".to_string(), "Dockerfile".to_string());
        language_map.insert("makefile".to_string(), "Makefile".to_string());
        
        // Solidity not included by default, would need custom syntax
        
        Ok(Self {
            syntax_set,
            theme_set,
            current_theme: "InspiredGitHub".to_string(),
            language_map,
        })
    }
    
    pub fn set_theme(&mut self, theme_name: &str) -> Result<()> {
        // Map Pandoc theme names to syntect theme names
        let syntect_theme = match theme_name {
            "pygments" => "base16-ocean.light",
            "kate" => "InspiredGitHub",
            "monochrome" => "base16-ocean.light",
            "breezedark" => "base16-ocean.dark",
            "espresso" => "base16-mocha.dark",
            "zenburn" => "base16-eighties.dark",
            "haddock" => "InspiredGitHub",
            "tango" => "InspiredGitHub",
            _ => theme_name,
        };
        
        if self.theme_set.themes.contains_key(syntect_theme) {
            self.current_theme = syntect_theme.to_string();
            Ok(())
        } else {
            // Default to InspiredGitHub if theme not found
            self.current_theme = "InspiredGitHub".to_string();
            Ok(())
        }
    }
    
    pub fn find_syntax(&self, language: &str) -> Option<&SyntaxReference> {
        // Try to find syntax by our mapped name first
        if let Some(syntax_name) = self.language_map.get(language) {
            self.syntax_set.find_syntax_by_name(syntax_name)
        } else {
            // Try direct lookup
            self.syntax_set.find_syntax_by_name(language)
                .or_else(|| self.syntax_set.find_syntax_by_extension(language))
                .or_else(|| {
                    // Try case-insensitive search
                    self.syntax_set.syntaxes().iter()
                        .find(|s| s.name.to_lowercase() == language.to_lowercase())
                })
        }
    }
    
    pub fn highlight_lines<'a>(&self, code: &'a str, language: Option<&str>) -> Vec<Vec<(Style, &'a str)>> {
        use syntect::easy::HighlightLines;
        
        let syntax = language
            .and_then(|lang| self.find_syntax(lang))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        
        let theme = &self.theme_set.themes[&self.current_theme];
        let mut h = HighlightLines::new(syntax, theme);
        
        let mut highlighted_lines = Vec::new();
        
        for line in LinesWithEndings::from(code) {
            let ranges = h.highlight_line(line, &self.syntax_set)
                .unwrap_or_else(|_| vec![(Style::default(), line)]);
            highlighted_lines.push(ranges);
        }
        
        highlighted_lines
    }
    
    pub fn highlight_to_html(&self, code: &str, language: Option<&str>) -> String {
        use syntect::html::{ClassedHTMLGenerator, ClassStyle};
        
        let syntax = language
            .and_then(|lang| self.find_syntax(lang))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        
        let mut generator = ClassedHTMLGenerator::new_with_class_style(
            syntax,
            &self.syntax_set,
            ClassStyle::Spaced
        );
        
        for line in LinesWithEndings::from(code) {
            let _ = generator.parse_html_for_line_which_includes_newline(line);
        }
        
        generator.finalize()
    }
    
    pub fn get_theme_background(&self) -> Option<(u8, u8, u8, u8)> {
        let theme = &self.theme_set.themes[&self.current_theme];
        theme.settings.background.map(|color| {
            (color.r, color.g, color.b, color.a)
        })
    }
    
    pub fn get_theme_foreground(&self) -> Option<(u8, u8, u8, u8)> {
        let theme = &self.theme_set.themes[&self.current_theme];
        theme.settings.foreground.map(|color| {
            (color.r, color.g, color.b, color.a)
        })
    }
}

/// Helper function to determine if syntax highlighting should be applied
pub fn should_highlight(language: Option<&str>) -> bool {
    language.is_some() && language != Some("text") && language != Some("plain")
}