mod animations;
mod app;
mod game;
mod leaderboard;
mod render;
mod stats;
mod persistence;
mod telemetry;
mod theme;

use app::App;
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "ukodus", about = "Terminal Sudoku game")]
struct Cli {
    /// Load a puzzle from an 81-character string or 8-character short code
    #[arg(long)]
    puzzle: Option<String>,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Run the app
    let result = run_app(&mut stdout, cli.puzzle.as_deref());

    // Restore terminal
    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }

    Ok(())
}

fn run_app(stdout: &mut io::Stdout, puzzle: Option<&str>) -> io::Result<()> {
    let mut app = App::new_with_puzzle(puzzle);
    let mut last_tick = Instant::now();

    loop {
        // Determine tick rate based on screen mode
        let tick_rate = app.get_tick_rate();

        // Render
        render::render(stdout, &mut app)?;
        stdout.flush()?;

        // Handle input with timeout for animation updates
        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout.min(Duration::from_millis(33)))? {
            if let Event::Key(key) = event::read()? {
                // Handle Ctrl+C
                if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
                    break;
                }

                match app.handle_key(key) {
                    app::AppAction::Continue => {}
                    app::AppAction::Quit => break,
                }
            }
        }

        // Tick animations and timer
        if last_tick.elapsed() >= tick_rate {
            app.tick();
            last_tick = Instant::now();
        }
    }

    Ok(())
}
