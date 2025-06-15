use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeMode {
    Light,
    Dark,
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::Light
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub background: String,
    pub text: String,
    pub keywords: String,
    pub strings: String,
    pub comments: String,
    pub numbers: String,
    pub functions: String,
    pub types: String,
    pub operators: String,
    pub line_numbers: String,
    pub header: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub mode: ThemeMode,
    pub colors: ColorScheme,
    pub font_size: f32,
    pub line_height: f32,
}

impl Theme {
    pub fn light() -> Self {
        Theme {
            mode: ThemeMode::Light,
            colors: ColorScheme {
                background: "#FFFFFF".to_string(),
                text: "#1a1a1a".to_string(),           // Dark gray text
                keywords: "#0066cc".to_string(),        // Blue keywords
                strings: "#008000".to_string(),         // Green strings
                comments: "#808080".to_string(),        // Gray comments
                numbers: "#800080".to_string(),         // Purple numbers
                functions: "#cc6600".to_string(),       // Orange functions
                types: "#008080".to_string(),           // Teal types
                operators: "#cc0000".to_string(),       // Red operators
                line_numbers: "#666666".to_string(),    // Medium gray line numbers
                header: "#000000".to_string(),          // Black headers
            },
            font_size: 10.0,
            line_height: 1.2,
        }
    }

    pub fn dark() -> Self {
        Theme {
            mode: ThemeMode::Dark,
            colors: ColorScheme {
                background: "#2d2d2d".to_string(),      // Dark gray background
                text: "#f0f0f0".to_string(),            // Light gray text
                keywords: "#66b3ff".to_string(),        // Light blue keywords
                strings: "#66ff66".to_string(),         // Light green strings
                comments: "#b3b3b3".to_string(),        // Light gray comments
                numbers: "#ff66ff".to_string(),         // Light purple numbers
                functions: "#ffcc66".to_string(),       // Light orange functions
                types: "#66ffcc".to_string(),           // Light teal types
                operators: "#ff6666".to_string(),       // Light red operators
                line_numbers: "#999999".to_string(),    // Gray line numbers
                header: "#ffffff".to_string(),          // White headers
            },
            font_size: 10.0,
            line_height: 1.2,
        }
    }

    pub fn from_mode(mode: ThemeMode) -> Self {
        match mode {
            ThemeMode::Light => Self::light(),
            ThemeMode::Dark => Self::dark(),
        }
    }

    /// Convert hex color to RGB values (0.0-1.0 range for PDF)
    pub fn hex_to_rgb(hex: &str) -> (f32, f32, f32) {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return (0.0, 0.0, 0.0); // Default to black for invalid colors
        }

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;

        (r, g, b)
    }

    /// Get color mapping for syntax highlighting
    pub fn get_color_map(&self) -> HashMap<String, (f32, f32, f32)> {
        let mut map = HashMap::new();
        
        map.insert("keyword".to_string(), Self::hex_to_rgb(&self.colors.keywords));
        map.insert("string".to_string(), Self::hex_to_rgb(&self.colors.strings));
        map.insert("comment".to_string(), Self::hex_to_rgb(&self.colors.comments));
        map.insert("number".to_string(), Self::hex_to_rgb(&self.colors.numbers));
        map.insert("function".to_string(), Self::hex_to_rgb(&self.colors.functions));
        map.insert("type".to_string(), Self::hex_to_rgb(&self.colors.types));
        map.insert("operator".to_string(), Self::hex_to_rgb(&self.colors.operators));
        map.insert("text".to_string(), Self::hex_to_rgb(&self.colors.text));
        
        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_rgb() {
        assert_eq!(Theme::hex_to_rgb("#FFFFFF"), (1.0, 1.0, 1.0));
        assert_eq!(Theme::hex_to_rgb("#000000"), (0.0, 0.0, 0.0));
        assert_eq!(Theme::hex_to_rgb("#FF0000"), (1.0, 0.0, 0.0));
    }

    #[test]
    fn test_theme_creation() {
        let light = Theme::light();
        assert!(matches!(light.mode, ThemeMode::Light));
        
        let dark = Theme::dark();
        assert!(matches!(dark.mode, ThemeMode::Dark));
    }

    #[test]
    fn test_color_map() {
        let theme = Theme::light();
        let color_map = theme.get_color_map();
        
        assert!(color_map.contains_key("keyword"));
        assert!(color_map.contains_key("string"));
        assert!(color_map.contains_key("comment"));
    }
}