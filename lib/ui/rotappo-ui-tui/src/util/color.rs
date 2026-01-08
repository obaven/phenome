//! Color helpers for animated UI effects.

use ratatui::style::Color;

use rotappo_domain::now_millis;

/// Animated hue rotation for accent colors.
///
/// # Examples
/// ```rust
/// use ratatui::style::Color;
/// use rotappo_ui_tui::util::animated_color;
///
/// let color = animated_color(1000, 0.0, 0.0);
/// assert_eq!(color, Color::Rgb(0, 0, 0));
/// ```
pub fn animated_color(phase_ms: u64, saturation: f32, value: f32) -> Color {
    let t = now_millis() as f32 / phase_ms.max(1) as f32;
    let hue = (t * 0.03) % 1.0;
    hsv_to_rgb(hue, saturation.clamp(0.0, 1.0), value.clamp(0.0, 1.0))
}

/// Compute a traveling glow effect for list entries.
///
/// # Examples
/// ```rust
/// use rotappo_ui_tui::util::traveling_glow;
///
/// assert!(traveling_glow(0, 0).is_none());
/// ```
pub fn traveling_glow(line_index: usize, total_lines: usize) -> Option<Color> {
    if total_lines == 0 {
        return None;
    }
    let t = now_millis() as f32 / 1000.0;
    let speed = 1.6;
    let center = (t * speed) % total_lines as f32;
    let distance = (line_index as f32 - center).abs();
    let band = 1.5;
    if distance > band {
        return None;
    }
    let fade = 1.0 - (distance / band);
    let hue = (t * 0.08 + line_index as f32 * 0.02) % 1.0;
    let value = (0.6 + 0.4 * fade).clamp(0.0, 1.0);
    Some(hsv_to_rgb(hue, 0.5, value))
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> Color {
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    let (r, g, b) = match i.rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };

    Color::Rgb(
        (r * 255.0) as u8,
        (g * 255.0) as u8,
        (b * 255.0) as u8,
    )
}
