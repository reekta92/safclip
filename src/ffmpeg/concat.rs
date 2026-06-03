use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
use uuid::Uuid;
use crate::{
    ffmpeg::{cut, probe},
    model::{MediaMetadata, Segment},
};

pub fn merge_segments_copy(source_path: &str, segments: &[Segment]) -> Result<String, String> {
    let source = Path::new(source_path);
    let metadata = probe::probe_media(source_path)?;
    merge_segments_copy_with_metadata(source, segments, &metadata)
}

pub fn merge_segments_copy_with_metadata(
    source: &Path,
    segments: &[Segment],
    metadata: &MediaMetadata,
) -> Result<String, String> {
    if segments.is_empty() {
        return Err("No segments were provided for merge".to_string());
    }

    if !source.exists() {
        return Err(format!("Source file does not exist: {}", source.display()));
    }

    let temp_dir = std::env::temp_dir().join(format!("safclip-tui-{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir)
        .map_err(|error| format!("Failed to create temporary directory: {error}"))?;

    let mut temp_files = Vec::with_capacity(segments.len());
    for (index, segment) in segments.iter().enumerate() {
        let (start_seconds, end_seconds) = cut::snap_bounds_to_keyframes(
            segment.start_seconds,
            segment.end_seconds,
            &metadata.keyframes_seconds,
            metadata.duration_seconds,
        )?;

        let extension = source
            .extension()
            .map(|ext| ext.to_string_lossy().to_string())
            .unwrap_or_else(|| "mp4".to_string());
        let temp_path = temp_dir.join(format!("part-{index:03}.{extension}"));

        cut::run_copy_cut_once(source, start_seconds, end_seconds, &temp_path)?;
        temp_files.push(temp_path);
    }

    let list_path = temp_dir.join("concat-list.txt");
    cut::write_concat_list(&list_path, &temp_files)?;

    let output = merged_output_path(source)?;
    let status = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-y")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(&list_path)
        .arg("-map")
        .arg("0")
        .arg("-c")
        .arg("copy")
        .arg(&output)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("Failed to run ffmpeg concat: {error}"))?;

    if !status.success() {
        let _ = fs::remove_dir_all(&temp_dir);
        return Err(format!(
            "ffmpeg concat failed for output {}",
            output.display()
        ));
    }

    let _ = fs::remove_dir_all(&temp_dir);

    Ok(output.to_string_lossy().to_string())
}

fn merged_output_path(source: &Path) -> Result<PathBuf, String> {
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

    let initial = parent.join(format!("{stem}-merged.{extension}"));
    if !initial.exists() {
        return Ok(initial);
    }

    for index in 1..1_000_000 {
        let candidate = parent.join(format!("{stem}-merged-{index:03}.{extension}"));
        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    Err("Unable to allocate merged output file name".to_string())
}

pub fn merge_segments_copy_with_metadata_progress(
    source: &Path,
    segments: &[Segment],
    metadata: &MediaMetadata,
    tx: Option<&std::sync::mpsc::Sender<crate::app::ExportMsg>>,
) -> Result<String, String> {
    if segments.is_empty() {
        return Err("No segments were provided for merge".to_string());
    }

    if !source.exists() {
        return Err(format!("Source file does not exist: {}", source.display()));
    }

    let temp_dir = std::env::temp_dir().join(format!("safclip-tui-{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir)
        .map_err(|error| format!("Failed to create temporary directory: {error}"))?;

    let mut temp_files = Vec::with_capacity(segments.len());
    for (index, segment) in segments.iter().enumerate() {
        let (start_seconds, end_seconds) = cut::snap_bounds_to_keyframes(
            segment.start_seconds,
            segment.end_seconds,
            &metadata.keyframes_seconds,
            metadata.duration_seconds,
        )?;

        let extension = source
            .extension()
            .map(|ext| ext.to_string_lossy().to_string())
            .unwrap_or_else(|| "mp4".to_string());
        let temp_path = temp_dir.join(format!("part-{index:03}.{extension}"));

        if let Some(sender) = tx {
            let prefix = format!("Cutting part {}/{}", index + 1, segments.len());
            crate::export::run_copy_cut_with_progress(source, start_seconds, end_seconds, &temp_path, sender, &prefix)?;
        } else {
            cut::run_copy_cut_once(source, start_seconds, end_seconds, &temp_path)?;
        }
        temp_files.push(temp_path);
    }

    let list_path = temp_dir.join("concat-list.txt");
    cut::write_concat_list(&list_path, &temp_files)?;

    if let Some(sender) = tx {
        let _ = sender.send(crate::app::ExportMsg::Progress("Merging clips...".to_string()));
    }

    let output = merged_output_path(source)?;
    let status = Command::new("ffmpeg")
        .arg("-hide_banner")
        .arg("-y")
        .arg("-f")
        .arg("concat")
        .arg("-safe")
        .arg("0")
        .arg("-i")
        .arg(&list_path)
        .arg("-map")
        .arg("0")
        .arg("-c")
        .arg("copy")
        .arg(&output)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("Failed to run ffmpeg concat: {error}"))?;

    if !status.success() {
        let _ = fs::remove_dir_all(&temp_dir);
        return Err(format!(
            "ffmpeg concat failed for output {}",
            output.display()
        ));
    }

    let _ = fs::remove_dir_all(&temp_dir);

    Ok(output.to_string_lossy().to_string())
}

