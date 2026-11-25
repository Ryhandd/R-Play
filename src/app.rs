use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::audio::AudioPlayer;
use crate::scanner::{self, Track};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

impl RepeatMode {
    pub fn next(self) -> Self {
        match self {
            RepeatMode::Off => RepeatMode::All,
            RepeatMode::All => RepeatMode::One,
            RepeatMode::One => RepeatMode::Off,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            RepeatMode::Off => "Off",
            RepeatMode::All => "All",
            RepeatMode::One => "One",
        }
    }
}

#[derive(Debug, Clone)]
pub enum BrowseEntry {
    SelectCurrent(PathBuf),
    Parent(Option<PathBuf>),
    Dir(PathBuf),
}

impl BrowseEntry {
    pub fn label(&self) -> String {
        match self {
            BrowseEntry::SelectCurrent(p) => format!("[✓] Use this folder: {}", p.display()),
            BrowseEntry::Parent(Some(_)) => ".. (Parent folder)".to_string(),
            BrowseEntry::Parent(None) => ".. (Drives)".to_string(),
            BrowseEntry::Dir(p) => {
                let name = p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| p.to_string_lossy().to_string());
                format!("[DIR] {name}")
            }
        }
    }
}

pub struct App {
    pub current_folder: PathBuf,
    pub tracks: Vec<Track>,
    pub filtered_indices: Vec<usize>,
    pub selected_index: usize,
    pub current_track_index: Option<usize>,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub search_mode: bool,
    pub search_query: String,
    pub folder_mode: bool,
    pub folder_typing: bool,
    pub folder_input: String,
    pub browse_dir: Option<PathBuf>,
    pub browse_entries: Vec<BrowseEntry>,
    pub browse_selected: usize,
    pub status_message: String,
    pub should_quit: bool,
    audio: Option<AudioPlayer>,
    rng_state: u64,
}

impl App {
    pub fn new(
        current_folder: PathBuf,
        tracks: Vec<Track>,
        shuffle: bool,
        repeat: RepeatMode,
    ) -> Self {
        let filtered_indices: Vec<usize> = (0..tracks.len()).collect();
        let (audio, status_message) = match AudioPlayer::new() {
            Ok(player) => {
                let msg = if tracks.is_empty() {
                    "No audio files found. Press 'O' to open a folder.".to_string()
                } else {
                    format!("Loaded {} tracks.", tracks.len())
                };
                (Some(player), msg)
            }
            Err(err) => (None, err),
        };

        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x5EED_CAFE);

        let mut app = Self {
            current_folder,
            tracks,
            filtered_indices,
            selected_index: 0,
            current_track_index: None,
            shuffle,
            repeat,
            search_mode: false,
            search_query: String::new(),
            folder_mode: false,
            folder_typing: false,
            folder_input: String::new(),
            browse_dir: None,
            browse_entries: Vec::new(),
            browse_selected: 0,
            status_message,
            should_quit: false,
            audio,
            rng_state: seed,
        };

        if !app.tracks.is_empty() {
            let start_idx = if app.shuffle {
                app.random_index(app.tracks.len())
            } else {
                0
            };
            app.play_track_at(start_idx);
        }

        app
    }

    pub fn current_track(&self) -> Option<&Track> {
        self.current_track_index.and_then(|i| self.tracks.get(i))
    }

    pub fn is_paused(&self) -> bool {
        self.audio.as_ref().map(|a| a.is_paused()).unwrap_or(true)
    }

    pub fn volume_percent(&self) -> u16 {
        let vol = self.audio.as_ref().map(|a| a.volume()).unwrap_or(1.0);
        (vol * 100.0).round() as u16
    }

    pub fn playback_position(&self) -> Duration {
        self.audio
            .as_ref()
            .map(|a| a.position())
            .unwrap_or(Duration::ZERO)
    }

    pub fn move_selection(&mut self, delta: isize) {
        if self.filtered_indices.is_empty() {
            self.selected_index = 0;
            return;
        }
        let len = self.filtered_indices.len() as isize;
        let next = (self.selected_index as isize + delta).rem_euclid(len);
        self.selected_index = next as usize;
    }

    pub fn play_selected(&mut self) {
        if self.tracks.is_empty() {
            self.enter_folder_mode();
            return;
        }
        if let Some(&track_idx) = self.filtered_indices.get(self.selected_index) {
            self.play_track_at(track_idx);
        }
    }

    pub fn toggle_pause(&mut self) {
        if self.current_track_index.is_none() {
            self.play_selected();
            return;
        }
        if let Some(audio) = &self.audio {
            let paused = audio.toggle_pause();
            self.status_message = if paused {
                "Paused".to_string()
            } else {
                "Playing".to_string()
            };
        }
    }

    pub fn next_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }
        let next_idx = if self.shuffle {
            self.random_index(self.tracks.len())
        } else {
            match self.current_track_index {
                Some(idx) if idx + 1 < self.tracks.len() => idx + 1,
                _ => 0,
            }
        };
        self.play_track_at(next_idx);
    }

    pub fn previous_track(&mut self) {
        if self.tracks.is_empty() {
            return;
        }
        let prev_idx = if self.shuffle {
            self.random_index(self.tracks.len())
        } else {
            match self.current_track_index {
                Some(0) | None => self.tracks.len() - 1,
                Some(idx) => idx - 1,
            }
        };
        self.play_track_at(prev_idx);
    }

    pub fn volume_up(&mut self) {
        if let Some(audio) = &mut self.audio {
            let vol = audio.adjust_volume(0.05);
            self.status_message = format!("Volume: {}%", (vol * 100.0).round() as u16);
        }
    }

    pub fn volume_down(&mut self) {
        if let Some(audio) = &mut self.audio {
            let vol = audio.adjust_volume(-0.05);
            self.status_message = format!("Volume: {}%", (vol * 100.0).round() as u16);
        }
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
        self.status_message = format!("Shuffle: {}", if self.shuffle { "On" } else { "Off" });
    }

    pub fn cycle_repeat(&mut self) {
        self.repeat = self.repeat.next();
        self.status_message = format!("Repeat: {}", self.repeat.label());
    }

    pub fn enter_search(&mut self) {
        if self.tracks.is_empty() {
            return;
        }
        self.search_mode = true;
        self.search_query.clear();
        self.apply_filter();
    }

    pub fn exit_search(&mut self, keep_filter: bool) {
        self.search_mode = false;
        if !keep_filter {
            self.search_query.clear();
            self.apply_filter();
        }
    }

    pub fn push_search_char(&mut self, ch: char) {
        self.search_query.push(ch);
        self.apply_filter();
    }

    pub fn pop_search_char(&mut self) {
        self.search_query.pop();
        self.apply_filter();
    }

    pub fn enter_folder_mode(&mut self) {
        self.folder_mode = true;
        self.folder_typing = false;
        self.folder_input.clear();

        let start = if self.current_folder.is_dir() {
            Some(self.current_folder.clone())
        } else {
            scanner::default_os_music_dir().filter(|p| p.is_dir())
        };
        self.set_browse_dir(start);
    }

    pub fn cancel_folder_mode(&mut self) {
        if self.folder_typing {
            self.folder_typing = false;
            self.folder_input.clear();
        } else {
            self.folder_mode = false;
            self.browse_entries.clear();
        }
    }

    pub fn start_folder_typing(&mut self) {
        self.folder_typing = true;
        self.folder_input = self
            .browse_dir
            .as_ref()
            .unwrap_or(&self.current_folder)
            .to_string_lossy()
            .to_string();
    }

    pub fn push_folder_char(&mut self, ch: char) {
        self.folder_input.push(ch);
    }

    pub fn pop_folder_char(&mut self) {
        self.folder_input.pop();
    }

    pub fn browse_move(&mut self, delta: isize) {
        if self.browse_entries.is_empty() {
            self.browse_selected = 0;
            return;
        }
        let len = self.browse_entries.len() as isize;
        self.browse_selected = (self.browse_selected as isize + delta).rem_euclid(len) as usize;
    }

    pub fn browse_activate(&mut self) {
        let Some(entry) = self.browse_entries.get(self.browse_selected).cloned() else {
            return;
        };
        match entry {
            BrowseEntry::SelectCurrent(path) => self.load_folder(path),
            BrowseEntry::Parent(parent) => self.set_browse_dir(parent),
            BrowseEntry::Dir(dir) => self.set_browse_dir(Some(dir)),
        }
    }

    pub fn browse_go_up(&mut self) {
        let Some(dir) = &self.browse_dir else {
            return;
        };
        let parent = dir.parent().map(|p| p.to_path_buf());
        self.set_browse_dir(parent);
    }

    pub fn browse_select_current(&mut self) {
        if let Some(dir) = self.browse_dir.clone() {
            self.load_folder(dir);
        }
    }

    pub fn confirm_folder(&mut self) {
        let raw = self
            .folder_input
            .trim()
            .trim_matches('"')
            .trim_matches('\'');
        if raw.is_empty() {
            self.cancel_folder_mode();
            return;
        }
        let path = PathBuf::from(raw);
        self.load_folder(path);
    }

    fn set_browse_dir(&mut self, dir: Option<PathBuf>) {
        self.browse_dir = dir.clone();
        self.browse_entries.clear();
        self.browse_selected = 0;

        match dir {
            Some(path) => {
                self.browse_entries
                    .push(BrowseEntry::SelectCurrent(path.clone()));
                let parent = path.parent().map(|p| p.to_path_buf());
                self.browse_entries.push(BrowseEntry::Parent(parent));
                for sub in scanner::list_subfolders(&path) {
                    self.browse_entries.push(BrowseEntry::Dir(sub));
                }
            }
            None => {
                for drive in scanner::list_drives() {
                    self.browse_entries.push(BrowseEntry::Dir(drive));
                }
            }
        }
    }

    fn load_folder(&mut self, path: PathBuf) {
        self.folder_mode = false;
        self.folder_typing = false;
        self.folder_input.clear();
        self.browse_entries.clear();

        let new_tracks = scanner::scan_directory(&path);

        if let Some(audio) = &self.audio {
            audio.stop();
        }

        self.current_folder = path.clone();
        self.tracks = new_tracks;
        self.current_track_index = None;
        self.selected_index = 0;
        self.search_query.clear();
        self.filtered_indices = (0..self.tracks.len()).collect();

        let _ = scanner::save_default_folder(&path);

        if self.tracks.is_empty() {
            self.status_message = format!("No audio files found in {}", path.display());
        } else {
            let start_idx = if self.shuffle {
                self.random_index(self.tracks.len())
            } else {
                0
            };
            self.play_track_at(start_idx);
        }
    }

    pub fn tick(&mut self) {
        let finished = self
            .audio
            .as_ref()
            .map(|a| a.is_empty() && !a.is_paused())
            .unwrap_or(false);

        if !finished || self.current_track_index.is_none() || self.tracks.is_empty() {
            return;
        }

        match self.repeat {
            RepeatMode::One => {
                if let Some(idx) = self.current_track_index {
                    self.play_track_at(idx);
                }
            }
            RepeatMode::All => self.next_track(),
            RepeatMode::Off => {
                if self.shuffle {
                    self.next_track();
                } else if let Some(idx) = self.current_track_index {
                    if idx + 1 < self.tracks.len() {
                        self.play_track_at(idx + 1);
                    } else {
                        self.current_track_index = None;
                        self.status_message = "Playback finished.".to_string();
                    }
                }
            }
        }
    }

    pub fn quit(&mut self) {
        if let Some(audio) = &self.audio {
            audio.stop();
        }
        self.should_quit = true;
    }

    fn play_track_at(&mut self, index: usize) {
        let Some(track) = self.tracks.get(index) else {
            return;
        };
        let path = track.path.clone();
        let display = track.display_name();

        let Some(audio) = &mut self.audio else {
            self.status_message = "Audio output unavailable.".to_string();
            return;
        };

        match audio.play_file(&path) {
            Ok(()) => {
                self.current_track_index = Some(index);
                self.status_message = format!("Playing: {display}");
            }
            Err(err) => {
                self.status_message = format!("Skipped ({display}): {err}");
                self.current_track_index = None;
            }
        }
    }

    fn apply_filter(&mut self) {
        let query = self.search_query.to_ascii_lowercase();
        self.filtered_indices = self
            .tracks
            .iter()
            .enumerate()
            .filter(|(_, track)| {
                query.is_empty()
                    || track.title.to_ascii_lowercase().contains(&query)
                    || track
                        .artist
                        .as_deref()
                        .unwrap_or("")
                        .to_ascii_lowercase()
                        .contains(&query)
                    || track
                        .album
                        .as_deref()
                        .unwrap_or("")
                        .to_ascii_lowercase()
                        .contains(&query)
            })
            .map(|(i, _)| i)
            .collect();
        self.selected_index = 0;
    }

    fn random_index(&mut self, len: usize) -> usize {
        // Simple xorshift64 to avoid adding a rand dependency.
        let mut x = self.rng_state;
        if x == 0 {
            x = 0x9E37_79B9_7F4A_7C15;
        }
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        (x as usize) % len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeat_mode_cycles_correctly() {
        assert_eq!(RepeatMode::Off.next(), RepeatMode::All);
        assert_eq!(RepeatMode::All.next(), RepeatMode::One);
        assert_eq!(RepeatMode::One.next(), RepeatMode::Off);
    }
}
