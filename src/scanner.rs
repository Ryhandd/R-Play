use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use lofty::prelude::*;
use walkdir::WalkDir;

const SUPPORTED_EXTENSIONS: [&str; 4] = ["mp3", "wav", "ogg", "flac"];

#[derive(Debug, Clone)]
pub struct Track {
    pub path: PathBuf,
    pub title: String,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Option<Duration>,
}

impl Track {
    pub fn display_name(&self) -> String {
        match &self.artist {
            Some(artist) if !artist.is_empty() => format!("{} - {}", artist, self.title),
            _ => self.title.clone(),
        }
    }
}

pub fn scan_directory(root: &Path) -> Vec<Track> {
    if !root.exists() || !root.is_dir() {
        return Vec::new();
    }

    let mut tracks: Vec<Track> = WalkDir::new(root)
        .follow_links(true)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file() && is_supported_audio(entry.path()))
        .map(|entry| read_track(entry.path()))
        .collect();

    tracks.sort_by(|a, b| a.path.cmp(&b.path));
    tracks
}

fn is_supported_audio(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| SUPPORTED_EXTENSIONS.contains(&ext.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

fn read_track(path: &Path) -> Track {
    let fallback_title = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Track")
        .to_string();

    let mut title = fallback_title;
    let mut artist = None;
    let mut album = None;
    let mut duration = None;

    if let Ok(tagged_file) = lofty::read_from_path(path) {
        let props_duration = tagged_file.properties().duration();
        if !props_duration.is_zero() {
            duration = Some(props_duration);
        }

        if let Some(tag) = tagged_file
            .primary_tag()
            .or_else(|| tagged_file.first_tag())
        {
            if let Some(t) = tag.title() {
                let t_str = t.trim();
                if !t_str.is_empty() {
                    title = t_str.to_string();
                }
            }
            if let Some(a) = tag.artist() {
                let a_str = a.trim();
                if !a_str.is_empty() {
                    artist = Some(a_str.to_string());
                }
            }
            if let Some(al) = tag.album() {
                let al_str = al.trim();
                if !al_str.is_empty() {
                    album = Some(al_str.to_string());
                }
            }
        }
    }

    Track {
        path: path.to_path_buf(),
        title,
        artist,
        album,
        duration,
    }
}

pub fn config_file_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(not(target_os = "windows"))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")));

    base.map(|dir| dir.join("rplay").join("default_folder.txt"))
}

pub fn default_os_music_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let home = std::env::var_os("USERPROFILE").map(PathBuf::from);
    #[cfg(not(target_os = "windows"))]
    let home = std::env::var_os("HOME").map(PathBuf::from);

    home.map(|dir| dir.join("Music"))
}

pub fn load_default_folder() -> Option<PathBuf> {
    let path = config_file_path()?;
    let content = fs::read_to_string(path).ok()?;
    let trimmed = content.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(PathBuf::from(trimmed))
    }
}

pub fn save_default_folder(folder: &Path) -> std::io::Result<()> {
    let Some(path) = config_file_path() else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not determine config directory",
        ));
    };
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, folder.to_string_lossy().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_extensions_are_case_insensitive() {
        assert!(is_supported_audio(Path::new("song.mp3")));
        assert!(is_supported_audio(Path::new("song.FLAC")));
        assert!(is_supported_audio(Path::new("song.Wav")));
        assert!(is_supported_audio(Path::new("song.ogg")));
        assert!(!is_supported_audio(Path::new("notes.txt")));
    }

    #[test]
    fn scan_nonexistent_directory_returns_empty() {
        let tracks = scan_directory(Path::new("this_folder_should_not_exist_rplay_test"));
        assert!(tracks.is_empty());
    }
}
