use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::time::Duration;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

pub struct AudioPlayer {
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sink: Sink,
    volume: f32,
}

impl AudioPlayer {
    pub fn new() -> Result<Self, String> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("Failed to initialize audio output: {e}"))?;
        let sink = Sink::try_new(&stream_handle)
            .map_err(|e| format!("Failed to create audio sink: {e}"))?;
        let volume = 1.0;
        sink.set_volume(volume);

        Ok(Self {
            _stream: stream,
            _stream_handle: stream_handle,
            sink,
            volume,
        })
    }

    pub fn play_file(&mut self, path: &Path) -> Result<(), String> {
        let file = File::open(path).map_err(|e| format!("Cannot open file: {e}"))?;
        let reader = BufReader::new(file);
        let source = Decoder::new(reader).map_err(|e| format!("Cannot decode audio: {e}"))?;

        self.sink.stop();
        self.sink.append(source);
        self.sink.set_volume(self.volume);
        self.sink.play();
        Ok(())
    }

    pub fn toggle_pause(&self) -> bool {
        if self.sink.is_paused() {
            self.sink.play();
            false
        } else {
            self.sink.pause();
            true
        }
    }

    pub fn stop(&self) {
        self.sink.stop();
    }

    pub fn is_paused(&self) -> bool {
        self.sink.is_paused()
    }

    pub fn is_empty(&self) -> bool {
        self.sink.empty()
    }

    pub fn position(&self) -> Duration {
        self.sink.get_pos()
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn adjust_volume(&mut self, delta: f32) -> f32 {
        self.volume = (self.volume + delta).clamp(0.0, 1.5);
        self.sink.set_volume(self.volume);
        self.volume
    }
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        self.sink.stop();
    }
}
