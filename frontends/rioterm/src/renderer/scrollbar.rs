//! Scrollbar rendering and state management
//!
//! This module provides scrollbar functionality for the terminal, including:
//! - Visual rendering of track and thumb
//! - Hit detection for mouse interaction
//! - State calculation based on scroll position and history size

use rio_backend::config::scrollbar::{Scrollbar as ScrollbarConfig, ScrollbarMode};
use rio_backend::sugarloaf::{Object, Quad};

/// Result of hit testing against the scrollbar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarHit {
    /// Not over the scrollbar
    None,
    /// Over the thumb (draggable part)
    Thumb,
    /// Over the track (above thumb)
    TrackAbove,
    /// Over the track (below thumb)
    TrackBelow,
}

/// Computed scrollbar state for a single context
#[derive(Debug, Clone)]
pub struct ScrollbarState {
    /// X position of the scrollbar (left edge)
    pub x: f32,
    /// Y position of the track (top edge)
    pub track_y: f32,
    /// Width of the scrollbar
    pub width: f32,
    /// Height of the track
    pub track_height: f32,
    /// Y position of the thumb (top edge, relative to window)
    pub thumb_y: f32,
    /// Height of the thumb
    pub thumb_height: f32,
    /// Whether the scrollbar should be visible
    pub visible: bool,
    /// Whether the thumb is being hovered
    pub hovered: bool,
}

impl Default for ScrollbarState {
    fn default() -> Self {
        Self {
            x: 0.0,
            track_y: 0.0,
            width: 8.0,
            track_height: 0.0,
            thumb_y: 0.0,
            thumb_height: 20.0,
            visible: false,
            hovered: false,
        }
    }
}

impl ScrollbarState {
    /// Calculate scrollbar state from terminal state
    ///
    /// # Arguments
    /// * `config` - Scrollbar configuration
    /// * `content_x` - X position where content starts
    /// * `content_y` - Y position where content starts (top)
    /// * `content_width` - Width of the content area
    /// * `content_height` - Height of the content area
    /// * `display_offset` - Current scroll offset (0 = bottom, history_size = top)
    /// * `history_size` - Total lines of scrollback history
    /// * `screen_lines` - Number of visible lines on screen
    /// * `scale` - Display scale factor
    pub fn calculate(
        config: &ScrollbarConfig,
        content_x: f32,
        content_y: f32,
        content_width: f32,
        content_height: f32,
        display_offset: usize,
        history_size: usize,
        screen_lines: usize,
        scale: f32,
    ) -> Self {
        // Determine visibility based on mode and history
        let visible = match config.mode {
            ScrollbarMode::Always => true,
            ScrollbarMode::Auto => history_size > 0,
            ScrollbarMode::Never => false,
        };

        if !visible {
            return Self {
                visible: false,
                ..Default::default()
            };
        }

        // All coordinates are in logical pixels (content_* are already in logical coords)
        let width = config.width;
        let x = content_x + content_width - width;
        let track_y = content_y;
        let track_height = content_height;

        // Calculate thumb dimensions
        let total_lines = history_size + screen_lines;
        let thumb_ratio = if total_lines > 0 {
            screen_lines as f32 / total_lines as f32
        } else {
            1.0
        };

        let min_thumb_height = config.thumb_min_height;
        let thumb_height = (track_height * thumb_ratio).max(min_thumb_height);

        // Calculate thumb position
        // display_offset: 0 = at bottom (latest), history_size = at top (oldest)
        // We want: when display_offset = 0, thumb at bottom; when display_offset = history_size, thumb at top
        let scroll_ratio = if history_size > 0 {
            display_offset as f32 / history_size as f32
        } else {
            0.0
        };

        // Scrollable range for thumb (total track minus thumb size)
        let scrollable_range = track_height - thumb_height;

        // Thumb Y: at top when scroll_ratio = 1, at bottom when scroll_ratio = 0
        let thumb_y = track_y + (1.0 - scroll_ratio) * scrollable_range;

        Self {
            x,
            track_y,
            width,
            track_height,
            thumb_y,
            thumb_height,
            visible,
            hovered: false,
        }
    }

    /// Check if a point (in physical pixels) is over the scrollbar
    ///
    /// # Arguments
    /// * `mouse_x` - Mouse X position in physical pixels
    /// * `mouse_y` - Mouse Y position in physical pixels
    /// * `scale` - Display scale factor
    pub fn hit_test(&self, mouse_x: f32, mouse_y: f32, scale: f32) -> ScrollbarHit {
        if !self.visible {
            return ScrollbarHit::None;
        }

        // Convert mouse coordinates from physical to logical pixels
        let logical_mouse_x = mouse_x / scale;
        let logical_mouse_y = mouse_y / scale;

        // Check if mouse is within scrollbar X bounds
        if logical_mouse_x < self.x || logical_mouse_x > self.x + self.width {
            return ScrollbarHit::None;
        }

        // Check if mouse is within track Y bounds
        if logical_mouse_y < self.track_y || logical_mouse_y > self.track_y + self.track_height {
            return ScrollbarHit::None;
        }

        // Check if over thumb
        if logical_mouse_y >= self.thumb_y && logical_mouse_y <= self.thumb_y + self.thumb_height {
            return ScrollbarHit::Thumb;
        }

        // Over track - determine if above or below thumb
        if logical_mouse_y < self.thumb_y {
            ScrollbarHit::TrackAbove
        } else {
            ScrollbarHit::TrackBelow
        }
    }

    /// Convert a Y coordinate to a scroll offset
    ///
    /// # Arguments
    /// * `y` - Y position in physical pixels
    /// * `history_size` - Total lines of scrollback history
    /// * `scale` - Display scale factor
    ///
    /// # Returns
    /// The scroll offset (0 = bottom, history_size = top)
    pub fn y_to_offset(&self, y: f32, history_size: usize, scale: f32) -> usize {
        if !self.visible || history_size == 0 {
            return 0;
        }

        // Convert mouse Y from physical to logical pixels
        let logical_y = y / scale;

        // Calculate the scrollable range (in logical pixels)
        let scrollable_range = self.track_height - self.thumb_height;
        if scrollable_range <= 0.0 {
            return 0;
        }

        // Calculate scroll ratio (0 = bottom, 1 = top)
        // thumb_y = track_y + (1 - scroll_ratio) * scrollable_range
        // So: scroll_ratio = 1 - (thumb_y - track_y) / scrollable_range
        let thumb_top_y = logical_y - self.thumb_height / 2.0;
        let relative_y = (thumb_top_y - self.track_y).clamp(0.0, scrollable_range);
        let scroll_ratio = 1.0 - (relative_y / scrollable_range);

        (scroll_ratio * history_size as f32).round() as usize
    }
}

/// Draw the scrollbar as Quad objects
///
/// # Arguments
/// * `state` - Computed scrollbar state (in logical pixel coordinates)
/// * `config` - Scrollbar configuration
/// * `hovered` - Whether the scrollbar thumb is being hovered
///
/// # Returns
/// Vector of Object::Quad for track and thumb
pub fn draw_scrollbar(
    state: &ScrollbarState,
    config: &ScrollbarConfig,
    hovered: bool,
) -> Vec<Object> {
    if !state.visible {
        return Vec::new();
    }

    let mut objects = Vec::with_capacity(2);

    // Draw track (coordinates are in logical pixels)
    // Using same pattern as create_border() which works correctly
    objects.push(Object::Quad(Quad {
        color: config.track_color,
        position: [state.x, state.track_y],
        size: [state.width, state.track_height],
        border_radius: [0.0, 0.0, 0.0, 0.0],
        shadow_blur_radius: 0.0,
        shadow_offset: [0.0, 0.0],
        shadow_color: [0.0, 0.0, 0.0, 0.0],
        border_color: [0.0, 0.0, 0.0, 0.0],
        border_width: 0.0,
    }));

    // Draw thumb with appropriate color based on hover state
    let thumb_color = if hovered {
        config.thumb_hover_color
    } else {
        config.thumb_color
    };

    objects.push(Object::Quad(Quad {
        color: thumb_color,
        position: [state.x, state.thumb_y],
        size: [state.width, state.thumb_height],
        border_radius: [0.0, 0.0, 0.0, 0.0],
        shadow_blur_radius: 0.0,
        shadow_offset: [0.0, 0.0],
        shadow_color: [0.0, 0.0, 0.0, 0.0],
        border_color: [0.0, 0.0, 0.0, 0.0],
        border_width: 0.0,
    }));

    objects
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> ScrollbarConfig {
        ScrollbarConfig::default()
    }

    #[test]
    fn test_scrollbar_hidden_when_no_history() {
        let state = ScrollbarState::calculate(
            &default_config(),
            0.0,
            0.0,
            800.0,
            600.0,
            0,
            0, // no history
            24,
            1.0,
        );
        assert!(!state.visible);
    }

    #[test]
    fn test_scrollbar_visible_with_history() {
        let state = ScrollbarState::calculate(
            &default_config(),
            0.0,
            0.0,
            800.0,
            600.0,
            0,
            100, // has history
            24,
            1.0,
        );
        assert!(state.visible);
    }

    #[test]
    fn test_scrollbar_always_mode() {
        let mut config = default_config();
        config.mode = ScrollbarMode::Always;

        let state = ScrollbarState::calculate(
            &config,
            0.0,
            0.0,
            800.0,
            600.0,
            0,
            0, // no history
            24,
            1.0,
        );
        assert!(state.visible);
    }

    #[test]
    fn test_scrollbar_never_mode() {
        let mut config = default_config();
        config.mode = ScrollbarMode::Never;

        let state = ScrollbarState::calculate(
            &config,
            0.0,
            0.0,
            800.0,
            600.0,
            0,
            100, // has history
            24,
            1.0,
        );
        assert!(!state.visible);
    }

    #[test]
    fn test_thumb_at_bottom_when_offset_zero() {
        let state = ScrollbarState::calculate(
            &default_config(),
            0.0,
            0.0,
            800.0,
            600.0,
            0, // at bottom
            100,
            24,
            1.0,
        );

        // Thumb should be at bottom of track
        let expected_thumb_bottom = state.track_y + state.track_height;
        let actual_thumb_bottom = state.thumb_y + state.thumb_height;
        assert!((expected_thumb_bottom - actual_thumb_bottom).abs() < 0.1);
    }

    #[test]
    fn test_thumb_at_top_when_offset_max() {
        let state = ScrollbarState::calculate(
            &default_config(),
            0.0,
            0.0,
            800.0,
            600.0,
            100, // at top (max offset)
            100,
            24,
            1.0,
        );

        // Thumb should be at top of track
        assert!((state.thumb_y - state.track_y).abs() < 0.1);
    }

    #[test]
    fn test_hit_test_thumb() {
        let state = ScrollbarState::calculate(
            &default_config(),
            0.0,
            0.0,
            800.0,
            600.0,
            50,
            100,
            24,
            1.0,
        );

        // Hit in the middle of the thumb
        let hit = state.hit_test(state.x + state.width / 2.0, state.thumb_y + state.thumb_height / 2.0, 1.0);
        assert_eq!(hit, ScrollbarHit::Thumb);
    }

    #[test]
    fn test_hit_test_track_above() {
        let state = ScrollbarState::calculate(
            &default_config(),
            0.0,
            0.0,
            800.0,
            600.0,
            0, // thumb at bottom
            100,
            24,
            1.0,
        );

        // Hit above the thumb
        let hit = state.hit_test(state.x + state.width / 2.0, state.track_y + 10.0, 1.0);
        assert_eq!(hit, ScrollbarHit::TrackAbove);
    }

    #[test]
    fn test_hit_test_outside() {
        let state = ScrollbarState::calculate(
            &default_config(),
            0.0,
            0.0,
            800.0,
            600.0,
            0,
            100,
            24,
            1.0,
        );

        // Hit outside scrollbar
        let hit = state.hit_test(100.0, 100.0, 1.0);
        assert_eq!(hit, ScrollbarHit::None);
    }
}
