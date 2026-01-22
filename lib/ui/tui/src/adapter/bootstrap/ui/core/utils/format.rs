use std::time::Duration;

pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let minutes = secs / 60;
    let seconds = secs % 60;
    if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

pub fn progress_bar(progress: f32, width: usize) -> String {
    let filled = ((progress.clamp(0.0, 1.0) * width as f32).round() as usize).min(width);
    let mut bar = String::new();
    for i in 0..width {
        bar.push(if i < filled { '#' } else { '.' });
    }
    bar
}

pub fn format_row(values: &[impl AsRef<str>], widths: &[usize; 4]) -> String {
    let mut out = String::new();
    for (idx, value) in values.iter().enumerate() {
        let width = widths.get(idx).copied().unwrap_or(10);
        let text = value.as_ref();
        out.push_str(&format!("{text:<width$}"));
    }
    out
}
