pub(super) fn format_bytes(bytes: f64) -> String {
    const KI: f64 = 1024.0;
    const MI: f64 = KI * 1024.0;
    const GI: f64 = MI * 1024.0;

    if bytes >= GI {
        format!("{:.2} GiB", bytes / GI)
    } else if bytes >= MI {
        format!("{:.2} MiB", bytes / MI)
    } else if bytes >= KI {
        format!("{:.2} KiB", bytes / KI)
    } else {
        format!("{:.0} B", bytes)
    }
}
