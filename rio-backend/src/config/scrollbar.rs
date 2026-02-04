use serde::{Deserialize, Serialize};

use super::colors::{deserialize_to_arr, ColorArray};

/// Default scrollbar width in pixels
fn default_width() -> f32 {
    8.0
}

/// Default minimum thumb height in pixels
fn default_thumb_min_height() -> f32 {
    20.0
}

/// Default border radius for scrollbar elements
fn default_border_radius() -> f32 {
    4.0
}

/// Default track color (semi-transparent dark)
fn default_track_color() -> ColorArray {
    [0.25, 0.25, 0.25, 0.25]
}

/// Default thumb color (semi-transparent light)
fn default_thumb_color() -> ColorArray {
    [0.5, 0.5, 0.5, 0.67]
}

/// Default thumb hover color (more opaque)
fn default_thumb_hover_color() -> ColorArray {
    [0.63, 0.63, 0.63, 0.87]
}

/// Scrollbar visibility mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ScrollbarMode {
    /// Always show the scrollbar
    Always,
    /// Show scrollbar only when there's scrollback history (default)
    #[default]
    Auto,
    /// Never show the scrollbar
    Never,
}

/// Scrollbar configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Scrollbar {
    /// Scrollbar visibility mode
    #[serde(default)]
    pub mode: ScrollbarMode,

    /// Scrollbar width in pixels
    #[serde(default = "default_width")]
    pub width: f32,

    /// Track background color
    #[serde(
        default = "default_track_color",
        rename = "track-color",
        deserialize_with = "deserialize_to_arr"
    )]
    pub track_color: ColorArray,

    /// Thumb color
    #[serde(
        default = "default_thumb_color",
        rename = "thumb-color",
        deserialize_with = "deserialize_to_arr"
    )]
    pub thumb_color: ColorArray,

    /// Thumb hover color (when mouse is over thumb)
    #[serde(
        default = "default_thumb_hover_color",
        rename = "thumb-hover-color",
        deserialize_with = "deserialize_to_arr"
    )]
    pub thumb_hover_color: ColorArray,

    /// Minimum thumb height in pixels
    #[serde(default = "default_thumb_min_height", rename = "thumb-min-height")]
    pub thumb_min_height: f32,

    /// Border radius for scrollbar elements
    #[serde(default = "default_border_radius", rename = "border-radius")]
    pub border_radius: f32,
}

impl Default for Scrollbar {
    fn default() -> Self {
        Scrollbar {
            mode: ScrollbarMode::default(),
            width: default_width(),
            track_color: default_track_color(),
            thumb_color: default_thumb_color(),
            thumb_hover_color: default_thumb_hover_color(),
            thumb_min_height: default_thumb_min_height(),
            border_radius: default_border_radius(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_scrollbar() {
        let scrollbar = Scrollbar::default();
        assert_eq!(scrollbar.mode, ScrollbarMode::Auto);
        assert_eq!(scrollbar.width, 8.0);
        assert_eq!(scrollbar.thumb_min_height, 20.0);
        assert_eq!(scrollbar.border_radius, 4.0);
    }

    #[test]
    fn test_scrollbar_mode_deserialize() {
        let toml_str = r#"
            mode = "always"
        "#;
        let scrollbar: Scrollbar = toml::from_str(toml_str).unwrap();
        assert_eq!(scrollbar.mode, ScrollbarMode::Always);

        let toml_str = r#"
            mode = "never"
        "#;
        let scrollbar: Scrollbar = toml::from_str(toml_str).unwrap();
        assert_eq!(scrollbar.mode, ScrollbarMode::Never);

        let toml_str = r#"
            mode = "auto"
        "#;
        let scrollbar: Scrollbar = toml::from_str(toml_str).unwrap();
        assert_eq!(scrollbar.mode, ScrollbarMode::Auto);
    }
}
