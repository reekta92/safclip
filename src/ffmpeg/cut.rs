use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

const CLIP_PREFIX: &str = "clip";

pub fn snap_bounds_to_keyframes(
    requested_start: f64,
    requested_end: f64,
    keyframes: &[f64],
    duration_seconds: f64,
) -> Result<(f64, f64), String> {
    if !requested_start.is_finite() || !requested_end.is_finite() {
        return Err("Segment bounds must be finite numbers".to_string());
    }

    if !duration_seconds.is_finite() || duration_seconds <= 0.0 {
        return Err("Media duration must be a positive finite number".to_string());
    }

    if requested_end <= requested_start {
        return Err("Segment end must be greater than start".to_string());
    }

    let start = requested_start.clamp(0.0, duration_seconds);
    let end = requested_end.clamp(0.0, duration_seconds);

    if keyframes.is_empty() {
        if end <= start {
            return Err("Segment has no positive duration after clamping".to_string());
        }
        return Ok((start, end));
    }

    let mut normalized = keyframes
        .iter()
        .copied()
        .filter(|value| value.is_finite() && *value >= 0.0 && *value <= duration_seconds)
        .collect::<Vec<_>>();

    normalized.push(0.0);
    normalized.push(duration_seconds);
    normalized.sort_by(|left, right| left.total_cmp(right));
    normalized.dedup_by(|left, right| (*left - *right).abs() <= f64::EPSILON);

    let mut snapped_start = normalized[0];
    for value in &normalized {
        if *value <= start {
            snapped_start = *value;
        } else {
            break;
        }
    }

    let mut snapped_end = *normalized.last().unwrap_or(&duration_seconds);
    for value in &normalized {
        if *value >= end {
            snapped_end = *value;
            break;
        }
    }

    if snapped_end <= snapped_start {
        let maybe_next = normalized.iter().copied().find(|value| *value > snapped_start);
        snapped_end = maybe_next.ok_or_else(|| {
            "Unable to find a keyframe-aligned end after snapped start".to_string()
        })?;
    }

    Ok((snapped_start, snapped_end))
}

pub fn build_copy_cut_args(
    input: &Path,
    start_seconds: f64,
    end_seconds: f64,
    output: &Path,
) -> Vec<String> {
    let duration_seconds = end_seconds - start_seconds;
    vec![
        "-hide_banner".to_string(),
        "-y".to_string(),
        "-ss".to_string(),
        format!("{start_seconds:.6}"),
        "-i".to_string(),
        input.to_string_lossy().to_string(),
        "-t".to_string(),
        format!("{duration_seconds:.6}"),
        "-map".to_string(),
        "0".to_string(),
        "-c".to_string(),
        "copy".to_string(),
        "-avoid_negative_ts".to_string(),
        "make_zero".to_string(),
        output.to_string_lossy().to_string(),
    ]
}

pub fn run_copy_cut_once(
    source: &Path,
    start_seconds: f64,
    end_seconds: f64,
    output_path: &Path,
) -> Result<(), String> {
    let args = build_copy_cut_args(source, start_seconds, end_seconds, output_path);

    let status = Command::new("ffmpeg")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("Failed to run ffmpeg: {error}"))?;

    if !status.success() {
        return Err(format!(
            "ffmpeg exited with non-zero status for output {}",
            output_path.display()
        ));
    }

    Ok(())
}

pub fn next_clip_output_path(source: &Path, start_index: u32) -> Result<PathBuf, String> {
    let parent = source
        .parent()
        .ok_or_else(|| "Source path has no parent directory".to_string())?;
    let stem = source
        .file_stem()
        .ok_or_else(|| "Source path has no file stem".to_string())?
        .to_string_lossy();
    let extension = source
        .extension()
        .map(|ext| ext.to_string_lossy().to_string())
        .unwrap_or_else(|| "mp4".to_string());

    let mut index = start_index;
    loop {
        let candidate = parent.join(format!("{stem}-{CLIP_PREFIX}-{index:03}.{extension}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
        index += 1;
        if index > 999_999 {
            return Err("Unable to allocate a unique output file name".to_string());
        }
    }
}

pub fn write_concat_list(list_path: &Path, files: &[PathBuf]) -> Result<(), String> {
    let mut lines = String::new();
    for file in files {
        let escaped = file
            .to_string_lossy()
            .replace('\\', "\\\\")
            .replace('\'', "\\'");
        lines.push_str("file '");
        lines.push_str(&escaped);
        lines.push_str("'\n");
    }

    fs::write(list_path, lines)
        .map_err(|error| format!("Failed writing concat list {}: {error}", list_path.display()))
}
