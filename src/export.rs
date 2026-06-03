use std::path::Path;
use std::process::{Command, Stdio};
use std::io::{BufReader, BufRead};
use std::sync::mpsc;
use crate::app::ExportMsg;
use crate::ffmpeg::probe::probe_media;
use crate::ffmpeg::cut::{snap_bounds_to_keyframes, build_copy_cut_args, next_clip_output_path};
use crate::ffmpeg::progress::extract_time_seconds;
use crate::model::Segment;

pub fn run_ffmpeg_with_progress(
    args: Vec<String>,
    total_duration: f64,
    tx: &mpsc::Sender<ExportMsg>,
    progress_prefix: &str,
) -> Result<(), String> {
    let mut child = Command::new("ffmpeg")
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn ffmpeg: {e}"))?;

    let stderr = child.stderr.take().ok_or("Failed to open stderr of ffmpeg")?;
    let reader = BufReader::new(stderr);

    for line_result in reader.lines() {
        if let Ok(line) = line_result {
            if let Some(time_seconds) = extract_time_seconds(&line) {
                if total_duration > 0.0 {
                    let percent = (time_seconds / total_duration * 100.0).clamp(0.0, 100.0);
                    let _ = tx.send(ExportMsg::Progress(format!("{}: {:.1}%", progress_prefix, percent)));
                }
            }
        }
    }

    let status = child.wait().map_err(|e| format!("ffmpeg failed to exit: {e}"))?;
    if !status.success() {
        return Err("ffmpeg exited with non-zero status".to_string());
    }

    Ok(())
}

pub fn run_copy_cut_with_progress(
    source: &Path,
    start_seconds: f64,
    end_seconds: f64,
    output_path: &Path,
    tx: &mpsc::Sender<ExportMsg>,
    progress_prefix: &str,
) -> Result<(), String> {
    let args = build_copy_cut_args(source, start_seconds, end_seconds, output_path);
    let duration = end_seconds - start_seconds;
    run_ffmpeg_with_progress(args, duration, tx, progress_prefix)
}

pub fn export_separate(
    source_path: &str,
    segments: &[Segment],
    tx: mpsc::Sender<ExportMsg>,
) -> Result<Vec<String>, String> {
    if segments.is_empty() {
        return Err("No segments to export".to_string());
    }
    let source = Path::new(source_path);
    if !source.exists() {
        return Err(format!("Source file not found: {}", source_path));
    }

    let _ = tx.send(ExportMsg::Progress("Probing metadata...".to_string()));
    let metadata = probe_media(source_path)?;
    let mut outputs = Vec::new();

    for (i, segment) in segments.iter().enumerate() {
        segment.validate_bounds()?;

        let (start, end) = snap_bounds_to_keyframes(
            segment.start_seconds,
            segment.end_seconds,
            &metadata.keyframes_seconds,
            metadata.duration_seconds,
        )?;

        let output_path = next_clip_output_path(source, i as u32 + 1)?;
        let prefix = format!("Exporting clip {}/{}", i + 1, segments.len());
        run_copy_cut_with_progress(source, start, end, &output_path, &tx, &prefix)?;
        outputs.push(output_path.to_string_lossy().to_string());
    }

    Ok(outputs)
}

pub fn export_merged(
    source_path: &str,
    segments: &[Segment],
    tx: mpsc::Sender<ExportMsg>,
) -> Result<String, String> {
    if segments.is_empty() {
        return Err("No segments to export".to_string());
    }
    let source = Path::new(source_path);
    if !source.exists() {
        return Err(format!("Source file not found: {}", source_path));
    }

    let _ = tx.send(ExportMsg::Progress("Probing metadata...".to_string()));
    let metadata = probe_media(source_path)?;

    // We can run the merged logic. But wait, concat demuxer merge in concat.rs runs Command standard
    // cutting. Let's make sure we have progress during cutting inside concat.rs or keep it simple.
    // Let's implement merge_segments_copy_with_progress in concat.rs!
    // Or we can just adapt concat.rs to take `tx`.
    // Let's modify concat.rs as well to support progress, or call it directly.
    // Let's check how concat.rs cuts:
    // It loops through segments and calls run_copy_cut_once.
    // If we adapt concat.rs to accept `tx: Option<&mpsc::Sender<ExportMsg>>`, we can show progress there too!
    // That is brilliant!
    crate::ffmpeg::concat::merge_segments_copy_with_metadata_progress(source, segments, &metadata, Some(&tx))
}
