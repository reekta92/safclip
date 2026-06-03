use std::path::{Path, PathBuf};
use std::fs;
use std::time::UNIX_EPOCH;
use serde::{Serialize, Deserialize};
use crate::model::Segment;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub version: u32,
    pub source_path: String,
    pub source_modified: u64,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionValidation {
    pub path_match: bool,
    pub modified_match: bool,
    pub source_exists: bool,
}

impl SessionValidation {
    pub fn is_valid(&self) -> bool {
        self.path_match && self.modified_match && self.source_exists
    }
}

pub fn session_path(source: &Path) -> PathBuf {
    let mut filename = source.file_name().unwrap_or_default().to_os_string();
    filename.push(".safclip.json");
    if let Some(parent) = source.parent() {
        parent.join(filename)
    } else {
        PathBuf::from(filename)
    }
}

pub fn save(source: &Path, segments: &[Segment]) -> Result<(), anyhow::Error> {
    let abs_source = fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf());
    let source_path_str = abs_source.to_string_lossy().into_owned();

    let metadata = fs::metadata(source)?;
    let modified = metadata.modified()?;
    let source_modified = modified.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();

    let session_data = SessionData {
        version: 1,
        source_path: source_path_str,
        source_modified,
        segments: segments.to_vec(),
    };

    let path = session_path(source);
    let json = serde_json::to_string_pretty(&session_data)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn load(source: &Path) -> Result<SessionData, anyhow::Error> {
    let path = session_path(source);
    let content = fs::read_to_string(path)?;
    let session_data: SessionData = serde_json::from_str(&content)?;
    Ok(session_data)
}

pub fn validate(session: &SessionData, source: &Path) -> SessionValidation {
    let source_exists = source.exists();

    let abs_source = fs::canonicalize(source).unwrap_or_else(|_| source.to_path_buf());
    let source_path_str = abs_source.to_string_lossy().into_owned();
    let path_match = session.source_path == source_path_str;

    let mut modified_match = false;
    if source_exists {
        if let Ok(metadata) = fs::metadata(source) {
            if let Ok(modified) = metadata.modified() {
                let source_modified = modified.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
                modified_match = session.source_modified == source_modified;
            }
        }
    }

    SessionValidation {
        path_match,
        modified_match,
        source_exists,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_path() {
        let path = Path::new("/path/to/video.mp4");
        let expected = Path::new("/path/to/video.mp4.safclip.json");
        assert_eq!(session_path(path), expected);

        let relative = Path::new("video.mp4");
        let expected_rel = Path::new("video.mp4.safclip.json");
        assert_eq!(session_path(relative), expected_rel);
    }

    #[test]
    fn test_save_load_validate() {
        let temp_dir = std::env::temp_dir();
        let video_file = temp_dir.join("test_video.mp4");
        fs::write(&video_file, "dummy content").unwrap();

        let segments = vec![
            Segment {
                id: "uuid-1".to_string(),
                start_seconds: 1.0,
                end_seconds: 5.5,
                label: Some("Introduction".to_string()),
            },
            Segment {
                id: "uuid-2".to_string(),
                start_seconds: 10.0,
                end_seconds: 20.0,
                label: None,
            },
        ];

        save(&video_file, &segments).unwrap();

        let s_path = session_path(&video_file);
        assert!(s_path.exists());

        let loaded = load(&video_file).unwrap();
        assert_eq!(loaded.version, 1);
        assert_eq!(loaded.segments, segments);

        let validation = validate(&loaded, &video_file);
        assert!(validation.is_valid());
        assert!(validation.source_exists);
        assert!(validation.path_match);
        assert!(validation.modified_match);

        let _ = fs::remove_file(&video_file);
        let _ = fs::remove_file(&s_path);
    }
}

