mod app;
mod audio;
mod scanner;
mod ui;

use std::io;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::{App, RepeatMode};

#[derive(Parser, Debug)]
#[command(
    name = "rplay",
    version,
    about = "Lightweight offline TUI music player"
)]
struct Cli {
    /// Path to the music folder (optional if a default folder is configured)
    path: Option<PathBuf>,

    /// Start playback in shuffle mode
    #[arg(long)]
    shuffle: bool,

    /// Enable repeat all mode
    #[arg(long)]
    repeat: bool,

    /// Save the given path as the default music folder
    #[arg(long)]
    set_default: bool,
}

struct TerminalGuard;

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let music_dir = match cli.path {
        Some(path) => {
            if cli.set_default {
                let _ = scanner::save_default_folder(&path);
            }
            path
        }
        None => match scanner::load_default_folder() {
            Some(default_path) => default_path,
            None => {
                eprintln!("Usage: rplay <MUSIC_FOLDER> [--shuffle] [--repeat] [--set-default]");
                std::process::exit(1);
            }
        },
    };

    let tracks = scanner::scan_directory(&music_dir);
    let repeat_mode = if cli.repeat {
        RepeatMode::All
    } else {
        RepeatMode::Off
    };

    let mut app = App::new(tracks, cli.shuffle, repeat_mode);

    let _guard = TerminalGuard::new()?;
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    let res = run_loop(&mut terminal, &mut app);
    app.quit();
    res
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::render(frame, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(app, key.code);
                }
            }
        }

        app.tick();
    }
    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode) {
    if app.search_mode {
        match code {
            KeyCode::Esc => app.exit_search(false),
            KeyCode::Enter => app.exit_search(true),
            KeyCode::Backspace => app.pop_search_char(),
            KeyCode::Char(c) => app.push_search_char(c),
            _ => {}
        }
        return;
    }

    match code {
        KeyCode::Char('q') | KeyCode::Char('Q') => app.quit(),
        KeyCode::Char(' ') => app.toggle_pause(),
        KeyCode::Char('n') | KeyCode::Char('N') => app.next_track(),
        KeyCode::Char('p') | KeyCode::Char('P') => app.previous_track(),
        KeyCode::Char('+') | KeyCode::Char('=') => app.volume_up(),
        KeyCode::Char('-') => app.volume_down(),
        KeyCode::Char('s') | KeyCode::Char('S') => app.toggle_shuffle(),
        KeyCode::Char('r') | KeyCode::Char('R') => app.cycle_repeat(),
        KeyCode::Char('/') => app.enter_search(),
        KeyCode::Up => app.move_selection(-1),
        KeyCode::Down => app.move_selection(1),
        KeyCode::Enter => app.play_selected(),
        _ => {}
    }
}
