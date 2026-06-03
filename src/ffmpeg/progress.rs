pub fn parse_ffmpeg_time_to_seconds(raw: &str) -> Option<f64> {
    let mut parts = raw.split(':');
    let hours = parts.next()?.parse::<f64>().ok()?;
    let minutes = parts.next()?.parse::<f64>().ok()?;
    let seconds = parts.next()?.parse::<f64>().ok()?;
    Some((hours * 3600.0) + (minutes * 60.0) + seconds)
}

pub fn extract_time_seconds(line: &str) -> Option<f64> {
    let marker = "time=";
    let marker_index = line.find(marker)? + marker.len();
    let after_marker = &line[marker_index..];
    let value = after_marker
        .split_ascii_whitespace()
        .next()
        .unwrap_or_default()
        .trim();

    if value.is_empty() {
        return None;
    }

    parse_ffmpeg_time_to_seconds(value)
}
