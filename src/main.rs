use std::io;
use std::time::{Duration, Instant};
use std::path::PathBuf;
use anyhow::Result;
use crossterm::{
    event::{self, EnableMouseCapture, DisableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{Terminal, backend::CrosstermBackend};

use safclip_controller::{app, input, ui};

fn main() -> Result<()> {
    // 1. Parse CLI arguments
    let args: Vec<String> = std::env::args().collect();
    let cli_path = if args.len() > 1 {
        Some(PathBuf::from(&args[1]))
    } else {
        None
    };

    // 2. Setup terminal raw mode and alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. Initialize state
    let mut state = app::AppState::new();
    let size = terminal.size()?;
    state.set_terminal_size(size.width, size.height);
    state.cli_source_path = cli_path;

    // Discover players initially
    state.refresh_players();

    // 4. Main event loop
    let mut last_poll = Instant::now();
    loop {
        // Poll MPRIS/ffmpeg state at ~10Hz
        let now = Instant::now();
        if now.duration_since(last_poll) >= Duration::from_millis(100) {
            state.poll_player();
            last_poll = now;
        }

        // Draw TUI only if dirty
        if state.is_dirty {
            terminal.draw(|f| ui::render(f, &mut state))?;
            state.is_dirty = false;
        }

        // Poll for input events with adaptive timeout
        // 8ms (125Hz) when playing or dirty to keep it snappy, 50ms when idle
        let timeout = if state.player_playing || state.is_dirty {
            Duration::from_millis(8)
        } else {
            Duration::from_millis(50)
        };
        if event::poll(timeout)? {
            match event::read()? {
                event::Event::Resize(w, h) => {
                    state.set_terminal_size(w, h);
                    state.is_dirty = true;
                }
                other => {
                    let action = input::handle_event(other);
                    state.apply_action(action);
                }
            }
        }

        if state.should_quit {
            break;
        }
    }

    // 5. Restore terminal state
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}
