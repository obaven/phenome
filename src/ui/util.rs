use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

use crate::adapters::bootstrappo::mapping;
use crate::runtime::{now_millis, CapabilityStatus, PlanStepStatus};

pub struct PlanLine {
    pub line: Line<'static>,
    pub step_index: Option<usize>,
}

pub fn spinner_frame() -> char {
    const FRAMES: [char; 4] = ['|', '/', '-', '\\'];
    let index = (now_millis() / 250) % FRAMES.len() as u64;
    FRAMES[index as usize]
}

pub fn animated_color(phase_ms: u64, saturation: f32, value: f32) -> Color {
    let t = now_millis() as f32 / phase_ms.max(1) as f32;
    let hue = (t * 0.03) % 1.0;
    hsv_to_rgb(hue, saturation.clamp(0.0, 1.0), value.clamp(0.0, 1.0))
}

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


pub fn plan_status_icon(status: PlanStepStatus) -> char {
    match status {
        PlanStepStatus::Running => spinner_frame(),
        PlanStepStatus::Succeeded => '+',
        PlanStepStatus::Failed => 'x',
        PlanStepStatus::Blocked => '!',
        PlanStepStatus::Pending => '.',
    }
}

pub fn capability_icon(status: CapabilityStatus) -> char {
    match status {
        CapabilityStatus::Ready => '+',
        CapabilityStatus::Degraded => '!',
        CapabilityStatus::Offline => 'x',
    }
}

pub fn format_age(timestamp_ms: u64) -> String {
    let now = crate::runtime::now_millis();
    let delta_ms = now.saturating_sub(timestamp_ms);
    let seconds = delta_ms / 1_000;
    if seconds < 60 {
        return format!("{}s ago", seconds);
    }
    let minutes = seconds / 60;
    if minutes < 60 {
        return format!("{}m ago", minutes);
    }
    let hours = minutes / 60;
    format!("{}h ago", hours)
}

pub fn plan_lines(snapshot: &crate::runtime::Snapshot) -> Vec<PlanLine> {
    let mut grouped: std::collections::BTreeMap<&str, Vec<(usize, &crate::runtime::PlanStep)>> =
        std::collections::BTreeMap::new();
    for (index, step) in snapshot.plan_steps.iter().enumerate() {
        grouped
            .entry(step.domain.as_str())
            .or_default()
            .push((index, step));
    }

    let mut lines = Vec::new();
    for (domain, steps) in grouped {
        lines.push(PlanLine {
            line: Line::from(Span::styled(
                format!("{} domain", domain),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            step_index: None,
        });
        for (index, step) in steps {
            let status_style = match step.status {
                PlanStepStatus::Succeeded => Style::default().fg(Color::Green),
                PlanStepStatus::Running => Style::default().fg(Color::Yellow),
                PlanStepStatus::Blocked => Style::default().fg(Color::Red),
                PlanStepStatus::Failed => {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                }
                PlanStepStatus::Pending => Style::default().fg(Color::Gray),
            };
            let pod_text = step
                .pod
                .as_deref()
                .map(|pod| format!(" pod: {}", pod))
                .unwrap_or_else(|| " pod: -".to_string());
            let line = Line::from(vec![
                Span::styled(
                    format!("[{} {:<9}]", plan_status_icon(step.status), step.status.as_str()),
                    status_style,
                ),
                Span::raw(" "),
                Span::styled(step.id.clone(), Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" "),
                Span::raw(step.kind.clone()),
                Span::styled(pod_text, Style::default().fg(Color::DarkGray)),
            ]);
            lines.push(PlanLine {
                line,
                step_index: Some(index),
            });
        }
    }
    lines
}

pub fn collect_problems(app: &crate::ui::app::App) -> Vec<String> {
    let mut problems = Vec::new();
    if let Some(live) = &app.backend.live_status {
        if let Some(error) = live.last_error() {
            problems.push(format!("kube: {}", error));
        }
        let health = live.health();
        problems.extend(mapping::health_problem_lines(&health));
        if live.cache().is_none() {
            problems.push("kube cache not ready".to_string());
        }
    } else {
        problems.push("live status disabled".to_string());
    }

    for step in &app.runtime.snapshot().plan_steps {
        if step.status == PlanStepStatus::Blocked {
            problems.push(format!("blocked: {} waiting on {:?}", step.id, step.depends_on));
        }
    }

    problems
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1]);

    horizontal[1]
}

pub fn anchored_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let width = area.width.saturating_mul(percent_x) / 100;
    let height = area.height.saturating_mul(percent_y) / 100;
    Rect::new(
        area.width.saturating_sub(width + 1),
        area.height.saturating_sub(height + 1),
        width,
        height,
    )
}

pub fn anchored_rect_with_offset(
    percent_x: u16,
    percent_y: u16,
    area: Rect,
    offset_x: i16,
    offset_y: i16,
) -> Rect {
    let base = anchored_rect(percent_x, percent_y, area);
    let x = base.x.saturating_add_signed(offset_x);
    let y = base.y.saturating_add_signed(offset_y);
    Rect::new(x, y, base.width, base.height)
}

fn line_width(line: &Line) -> u16 {
    line.spans
        .iter()
        .map(|span| span.content.len() as u16)
        .sum()
}

fn tooltip_size(area: Rect, lines: &[Line], max_width_pct: u16, max_height_pct: u16) -> (u16, u16) {
    let max_line = lines.iter().map(line_width).max().unwrap_or(0);
    let mut width = max_line.saturating_add(4);
    let mut height = (lines.len() as u16).saturating_add(2);
    let max_width = area.width.saturating_mul(max_width_pct) / 100;
    let max_height = area.height.saturating_mul(max_height_pct) / 100;
    width = width.min(max_width.max(12));
    height = height.min(max_height.max(6));
    (width, height)
}

pub fn tooltip_rect_for_mouse(
    area: Rect,
    mouse_pos: Option<(u16, u16)>,
    lines: &[Line],
    max_width_pct: u16,
    max_height_pct: u16,
) -> Rect {
    let (mut width, mut height) = tooltip_size(area, lines, max_width_pct, max_height_pct);
    let (mouse_x, mouse_y) = mouse_pos.unwrap_or((
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    ));
    let mut x = mouse_x.saturating_add(2);
    let mut y = mouse_y.saturating_add(1);
    let right_edge = area.x.saturating_add(area.width);
    let bottom_edge = area.y.saturating_add(area.height);

    if x + width > right_edge {
        x = mouse_x.saturating_sub(width.saturating_add(2));
    }
    if y + height > bottom_edge {
        y = mouse_y.saturating_sub(height.saturating_add(1));
    }

    if x < area.x {
        x = area.x;
    }
    if y < area.y {
        y = area.y;
    }
    if x + width > right_edge {
        width = right_edge.saturating_sub(x);
    }
    if y + height > bottom_edge {
        height = bottom_edge.saturating_sub(y);
    }

    Rect::new(x, y, width, height)
}

pub fn tooltip_rect_in_corner(
    area: Rect,
    lines: &[Line],
    max_width_pct: u16,
    max_height_pct: u16,
    margin_x: u16,
    margin_y: u16,
) -> Rect {
    let (width, height) = tooltip_size(area, lines, max_width_pct, max_height_pct);
    let right_edge = area.x.saturating_add(area.width);
    let bottom_edge = area.y.saturating_add(area.height);
    let x = right_edge
        .saturating_sub(width.saturating_add(margin_x).max(1));
    let y = bottom_edge
        .saturating_sub(height.saturating_add(margin_y).max(1));
    Rect::new(x.max(area.x), y.max(area.y), width, height)
}
