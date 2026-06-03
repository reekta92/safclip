use std::path::Path;
use std::process::Command;
use serde::Deserialize;
use crate::model::MediaMetadata;

#[derive(Debug, Deserialize)]
struct ProbeJson {
    format: Option<ProbeFormat>,
}

#[derive(Debug, Deserialize)]
struct ProbeFormat {
    duration: Option<String>,
    format_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct KeyframeJson {
    frames: Option<Vec<KeyframeFrame>>,
}

#[derive(Debug, Deserialize)]
struct KeyframeFrame {
    pts_time: Option<String>,
}

pub fn probe_media(source_path: &str) -> Result<MediaMetadata, String> {
    let source = Path::new(source_path);
    if !source.exists() {
        return Err(format!("Source file does not exist: {}", source_path));
    }

    let probe_output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-show_format")
        .arg("-of")
        .arg("json")
        .arg(source)
        .output()
        .map_err(|error| format!("Failed to run ffprobe: {error}"))?;

    if !probe_output.status.success() {
        return Err(format!(
            "ffprobe failed: {}",
            String::from_utf8_lossy(&probe_output.stderr)
        ));
    }

    let parsed: ProbeJson = serde_json::from_slice(&probe_output.stdout)
        .map_err(|error| format!("Invalid ffprobe JSON output: {error}"))?;

    let duration_seconds = parsed
        .format
        .as_ref()
        .and_then(|format| format.duration.as_ref())
        .and_then(|duration| duration.parse::<f64>().ok())
        .filter(|duration| duration.is_finite() && *duration > 0.0)
        .ok_or_else(|| "ffprobe did not provide a valid duration".to_string())?;

    let mut keyframes_seconds = probe_keyframes(source)?;

    // Ensure start and end are considered keyframes for boundary calculations
    keyframes_seconds.push(0.0);
    keyframes_seconds.push(duration_seconds);

    // Clean up keyframes: filter, sort, and deduplicate
    keyframes_seconds.retain(|value| value.is_finite() && *value >= 0.0 && *value <= duration_seconds);
    keyframes_seconds.sort_by(|left, right| left.total_cmp(right));
    keyframes_seconds.dedup_by(|left, right| (*left - *right).abs() <= f64::EPSILON);

    Ok(MediaMetadata {
        source_path: source_path.to_string(),
        duration_seconds,
        format_name: parsed.format.and_then(|format| format.format_name),
        keyframes_seconds,
    })
}

pub fn probe_keyframes(source: &Path) -> Result<Vec<f64>, String> {
    let output = Command::new("ffprobe")
        .arg("-v")
        .arg("error")
        .arg("-skip_frame")
        .arg("nokey")
        .arg("-select_streams")
        .arg("v:0")
        .arg("-show_entries")
        .arg("frame=pts_time")
        .arg("-of")
        .arg("json")
        .arg(source)
        .output()
        .map_err(|error| format!("Failed to run ffprobe for keyframes: {error}"))?;

    if !output.status.success() {
        return Err(format!(
            "ffprobe keyframe query failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    let parsed: KeyframeJson = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("Invalid keyframe JSON output: {error}"))?;

    let frames = parsed.frames.unwrap_or_default();
    let mut keyframes = Vec::with_capacity(frames.len());

    for frame in frames {
        if let Some(pts_time) = frame.pts_time {
            if let Ok(value) = pts_time.parse::<f64>() {
                keyframes.push(value);
            }
        }
    }

    Ok(keyframes)
}
