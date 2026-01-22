use ratatui::text::Line;

pub fn table_widths(total_width: u16) -> [usize; 4] {
    let total = total_width.max(20) as usize;
    let component = total * 30 / 100;
    let status = total * 30 / 100;
    let time = total * 15 / 100;
    let progress = total - component - status - time;
    [
        component.max(12),
        status.max(12),
        time.max(6),
        progress.max(8),
    ]
}

pub fn slice_lines(lines: &[Line<'static>], start: usize, height: usize) -> Vec<Line<'static>> {
    if lines.is_empty() {
        return Vec::new();
    }
    let start = start.min(lines.len().saturating_sub(1));
    let end = (start + height).min(lines.len());
    lines[start..end].to_vec()
}
