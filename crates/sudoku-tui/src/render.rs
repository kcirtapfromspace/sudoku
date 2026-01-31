use crate::animations::particles::hue_to_rgb;
use crate::animations::CelebrationManager;
use crate::app::{App, InputMode, MenuState, ScreenState, MAX_MISTAKES};
use crate::stats::{format_time, GameResult};
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    execute,
    style::{Color, Print, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};
use std::io;
use sudoku_core::Position;

pub fn render(stdout: &mut io::Stdout, app: &mut App) -> io::Result<()> {
    let (term_width, term_height) = terminal::size()?;

    execute!(stdout, Hide)?;

    match app.screen_state {
        // For animation screens, don't clear - we redraw everything anyway
        ScreenState::Win => render_win_screen(stdout, app, term_width, term_height)?,
        ScreenState::Lose => render_lose_screen(stdout, app, term_width, term_height)?,
        ScreenState::Stats => {
            execute!(stdout, Clear(ClearType::All))?;
            render_stats_screen(stdout, app, term_width, term_height)?;
        }
        ScreenState::Leaderboard => {
            execute!(stdout, Clear(ClearType::All))?;
            render_leaderboard_screen(stdout, app, term_width, term_height)?;
        }
        ScreenState::History => {
            execute!(stdout, Clear(ClearType::All))?;
            render_history_screen(stdout, app, term_width, term_height)?;
        }
        ScreenState::Playing => {
            // Only clear for playing mode to avoid flicker during animations
            execute!(stdout, Clear(ClearType::All))?;
            render_game_screen(stdout, app, term_width, term_height)?;
        }
    }

    execute!(stdout, Show)?;
    Ok(())
}

fn render_game_screen(
    stdout: &mut io::Stdout,
    app: &App,
    term_width: u16,
    term_height: u16,
) -> io::Result<()> {
    // Grid dimensions: 37 chars wide x 19 tall
    // Each cell is 3 chars wide, plus borders
    // Row: | X | X | X || X | X | X || X | X | X |
    //      1 + (3*3 + 2) * 3 + 2 = 1 + 11*3 + 2 = 36... let me recalc
    // Actually: "|  X  X  X |  X  X  X |  X  X  X |"
    // = 1 + 9 + 1 + 9 + 1 + 9 + 1 = 31 chars for cells + borders

    let grid_width: u16 = 37;
    let grid_height: u16 = 19;

    // Center the grid horizontally, leave room for info panel
    let total_width = grid_width + 25; // grid + gap + info panel
    let start_x = if term_width > total_width {
        (term_width - total_width) / 2
    } else {
        1
    };

    let start_y = if term_height > grid_height + 12 { 2 } else { 1 };

    render_grid(stdout, app, start_x, start_y)?;

    let info_x = start_x + grid_width + 3;
    render_info_panel(stdout, app, info_x, start_y)?;

    let controls_y = start_y + grid_height + 1;
    render_controls(stdout, app, start_x, controls_y)?;

    if let Some(ref msg) = app.message {
        render_message(stdout, app, msg, term_width)?;
    }

    if app.menu != MenuState::None {
        render_menu(stdout, app, term_width, term_height)?;
    }

    if let Some(ref hint) = app.current_hint {
        render_hint(stdout, app, &hint.explanation, term_width, term_height)?;
    }

    Ok(())
}

fn render_grid(stdout: &mut io::Stdout, app: &App, x: u16, y: u16) -> io::Result<()> {
    let theme = &app.theme;

    // Grid design:
    // +---+---+---+---+---+---+---+---+---+
    // | 5 | 3 | . | . | 7 | . | . | . | . |
    // +---+---+---+---+---+---+---+---+---+
    // Each cell is 3 chars: " X "
    // Border chars: + and - and |

    execute!(stdout, SetBackgroundColor(theme.bg))?;

    // Top border (thick - uses box_border for visibility)
    // Check if row 0 is celebrating
    let row0_intensity = app.celebrations.row_intensity(0);
    let top_border_color = CelebrationManager::throb_color(theme.box_border, row0_intensity);
    execute!(
        stdout,
        MoveTo(x, y),
        SetForegroundColor(top_border_color),
        Print("+===+===+===+===+===+===+===+===+===+")
    )?;

    for row in 0..9 {
        let cell_y = y + 1 + row as u16 * 2;

        // Get row celebration intensity
        let row_intensity = app.celebrations.row_intensity(row);

        // Cell row
        execute!(stdout, MoveTo(x, cell_y))?;

        for col in 0..9 {
            // Get column celebration intensity
            let col_intensity = app.celebrations.column_intensity(col);
            let border_intensity = row_intensity.max(col_intensity);

            // Left border - thick borders at 3x3 boundaries
            let base_border_color = if col % 3 == 0 {
                theme.box_border
            } else {
                theme.border
            };
            let border_color = CelebrationManager::throb_color(base_border_color, border_intensity);

            if col % 3 == 0 {
                execute!(stdout, SetForegroundColor(border_color), Print("║"))?;
            } else {
                execute!(stdout, SetForegroundColor(border_color), Print("│"))?;
            }

            let pos = Position::new(row, col);
            render_cell(stdout, app, pos)?;
        }
        // Right border (thick) - use row intensity for right border
        let right_border_color = CelebrationManager::throb_color(theme.box_border, row_intensity);
        execute!(stdout, SetForegroundColor(right_border_color), Print("║"))?;

        // Horizontal separator
        let sep_y = cell_y + 1;
        execute!(stdout, MoveTo(x, sep_y))?;

        // Get intensities for the row below this separator
        let next_row = row + 1;
        let row_below_intensity = if next_row < 9 {
            app.celebrations.row_intensity(next_row)
        } else {
            0.0
        };
        let sep_intensity = row_intensity.max(row_below_intensity);

        if row == 8 {
            // Bottom border (thick - highlighted)
            let bottom_color = CelebrationManager::throb_color(theme.box_border, row_intensity);
            execute!(
                stdout,
                SetForegroundColor(bottom_color),
                Print("+===+===+===+===+===+===+===+===+===+")
            )?;
        } else if (row + 1) % 3 == 0 {
            // Box separator (thick - highlighted)
            let box_sep_color = CelebrationManager::throb_color(theme.box_border, sep_intensity);
            execute!(
                stdout,
                SetForegroundColor(box_sep_color),
                Print("+===+===+===+===+===+===+===+===+===+")
            )?;
        } else {
            // Regular separator (thinner color)
            let sep_color = CelebrationManager::throb_color(theme.border, sep_intensity);
            execute!(
                stdout,
                SetForegroundColor(sep_color),
                Print("+---+---+---+---+---+---+---+---+---+")
            )?;
        }
    }

    Ok(())
}

fn render_cell(stdout: &mut io::Stdout, app: &App, pos: Position) -> io::Result<()> {
    let theme = &app.theme;
    let game = &app.game;
    let cell = game.grid().cell(pos);
    let is_cursor = pos == app.cursor;
    let is_highlighted = app.is_highlighted(pos);
    let has_same_value = app.has_same_value(pos);
    let has_conflict = game.has_conflict(pos);

    // Calculate celebration intensity for this cell
    let row_intensity = app.celebrations.row_intensity(pos.row);
    let col_intensity = app.celebrations.column_intensity(pos.col);
    let box_idx = (pos.row / 3) * 3 + (pos.col / 3);
    let box_intensity = app.celebrations.box_intensity(box_idx);
    // Take the maximum intensity from any active celebration affecting this cell
    let celebration_intensity = row_intensity.max(col_intensity).max(box_intensity);

    // Background color
    let mut bg = if is_cursor {
        theme.selected_bg
    } else if has_same_value && !cell.is_empty() {
        Color::Rgb {
            r: 60,
            g: 60,
            b: 100,
        }
    } else if is_highlighted {
        theme.highlight_bg
    } else {
        theme.bg
    };

    // Apply celebration throbbing to background
    if celebration_intensity > 0.0 {
        bg = CelebrationManager::throb_color(bg, celebration_intensity);
    }

    // Foreground color
    let mut fg = if has_conflict {
        theme.error
    } else if cell.is_given() {
        theme.given
    } else if cell.is_filled() {
        theme.filled
    } else {
        theme.candidate
    };

    // Apply celebration throbbing to foreground
    if celebration_intensity > 0.0 && !has_conflict {
        fg = CelebrationManager::throb_color(fg, celebration_intensity * 0.5);
    }

    execute!(stdout, SetBackgroundColor(bg), SetForegroundColor(fg))?;

    // Cell content: 3 chars " X "
    if let Some(value) = cell.value() {
        execute!(stdout, Print(format!(" {} ", value)))?;
    } else {
        let candidates = cell.candidates();
        let count = candidates.count();
        if count == 0 {
            // No candidates - check if we should show the valid candidate as a hint
            if app.show_naked_singles {
                let valid = app.game.grid().get_candidates(pos);
                if valid.count() == 1 {
                    let val = valid.single_value().unwrap();
                    execute!(
                        stdout,
                        SetForegroundColor(Color::DarkGrey),
                        Print(format!(" {} ", val))
                    )?;
                } else {
                    execute!(stdout, SetForegroundColor(Color::DarkGrey), Print(" · "))?;
                }
            } else {
                execute!(stdout, SetForegroundColor(Color::DarkGrey), Print(" · "))?;
            }
        } else if count == 1 {
            // Single note - show it (this is user's own note, always show)
            let val = candidates.single_value().unwrap();
            execute!(stdout, Print(format!(" {} ", val)))?;
        } else {
            // Multiple notes - show asterisk
            execute!(stdout, Print(" * "))?;
        }
    }

    Ok(())
}

fn render_info_panel(stdout: &mut io::Stdout, app: &App, x: u16, y: u16) -> io::Result<()> {
    let theme = &app.theme;
    let game = &app.game;

    execute!(stdout, SetBackgroundColor(theme.bg))?;

    // Title
    execute!(
        stdout,
        MoveTo(x, y),
        SetForegroundColor(theme.key),
        Print("═══ SUDOKU ═══")
    )?;

    // Time
    execute!(
        stdout,
        MoveTo(x, y + 2),
        SetForegroundColor(theme.info),
        Print(format!("Time: {:>10}", game.elapsed_string()))
    )?;

    // Difficulty
    execute!(
        stdout,
        MoveTo(x, y + 4),
        SetForegroundColor(theme.info),
        Print(format!("Level: {:>9}", format!("{}", game.difficulty())))
    )?;

    // Mode
    let mode_str = match app.mode {
        InputMode::Normal => "Normal",
        InputMode::Candidate => "Notes ",
        InputMode::Menu => "Menu  ",
    };
    let mode_color = if app.mode == InputMode::Candidate {
        Color::Cyan
    } else {
        theme.fg
    };
    execute!(
        stdout,
        MoveTo(x, y + 6),
        SetForegroundColor(theme.info),
        Print("Mode: "),
        SetForegroundColor(mode_color),
        Print(format!("{:>10}", mode_str))
    )?;

    // Mistakes
    let mistakes_color = if game.mistakes() >= MAX_MISTAKES - 1 {
        theme.error
    } else if game.mistakes() > 0 {
        Color::Yellow
    } else {
        theme.info
    };
    execute!(
        stdout,
        MoveTo(x, y + 8),
        SetForegroundColor(mistakes_color),
        Print(format!(
            "Mistakes: {:>6}",
            format!("{}/{}", game.mistakes(), MAX_MISTAKES)
        ))
    )?;

    // Hints
    execute!(
        stdout,
        MoveTo(x, y + 10),
        SetForegroundColor(theme.info),
        Print(format!("Hints used: {:>4}", game.hints_used()))
    )?;

    // Separator
    execute!(
        stdout,
        MoveTo(x, y + 12),
        SetForegroundColor(theme.border),
        Print("────────────────")
    )?;

    // Number completion indicator
    let completed = game.completed_numbers();
    execute!(
        stdout,
        MoveTo(x, y + 14),
        SetForegroundColor(theme.info),
        Print("Numbers: ")
    )?;
    for (i, &is_completed) in completed.iter().enumerate() {
        let num = (i + 1) as u8;
        if is_completed {
            execute!(
                stdout,
                SetForegroundColor(theme.success),
                Print(format!("{}", num))
            )?;
        } else {
            execute!(
                stdout,
                SetForegroundColor(theme.border),
                Print(format!("{}", num))
            )?;
        }
    }

    // Current cell
    let pos = app.cursor;
    let cell = game.grid().cell(pos);
    execute!(
        stdout,
        MoveTo(x, y + 16),
        SetForegroundColor(theme.info),
        Print(format!("Cell: Row {} Col {}", pos.row + 1, pos.col + 1))
    )?;

    if cell.is_empty() {
        if app.show_suggestions {
            let valid = game.grid().get_candidates(pos);
            let valid_str: String = valid
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            execute!(
                stdout,
                MoveTo(x, y + 17),
                SetForegroundColor(Color::Green),
                Print(format!("Valid: {:>9}", valid_str))
            )?;
        } else {
            execute!(
                stdout,
                MoveTo(x, y + 17),
                SetForegroundColor(theme.border),
                Print("Valid:   (v=show)")
            )?;
        }

        let notes = cell.candidates();
        let notes_str: String = notes
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        execute!(
            stdout,
            MoveTo(x, y + 18),
            SetForegroundColor(theme.candidate),
            Print(format!("Notes: {:>9}", notes_str))
        )?;
    } else {
        execute!(
            stdout,
            MoveTo(x, y + 17),
            SetForegroundColor(theme.bg),
            Print("                 ")
        )?;
        execute!(stdout, MoveTo(x, y + 18), Print("                 "))?;
    }

    Ok(())
}

fn render_controls(stdout: &mut io::Stdout, app: &App, x: u16, y: u16) -> io::Result<()> {
    let theme = &app.theme;

    execute!(stdout, SetBackgroundColor(theme.bg))?;

    let controls = [
        ("hjkl/Arrows", "Move"),
        ("1-9", "Set/Note"),
        ("0/Del", "Clear"),
        ("c", "Notes mode"),
        ("f/F", "Fill notes"),
        ("x/X", "Clear notes"),
        ("v", "Valid hints"),
        ("g", "Ghost hints"),
        ("n", "New game"),
        ("?/!", "Hint"),
        ("u", "Undo"),
        ("i", "Stats"),
        ("b", "Leaderboard"),
        ("H", "History"),
        ("t", "Theme"),
        ("q", "Quit"),
    ];

    // Display in 4 columns (4 items each)
    for (i, (key, desc)) in controls.iter().enumerate() {
        let col = i / 4;
        let row = i % 4;
        let cx = x + (col as u16) * 17;
        let cy = y + row as u16;

        execute!(
            stdout,
            MoveTo(cx, cy),
            SetForegroundColor(theme.key),
            Print(format!("{:>8}", key)),
            SetForegroundColor(theme.info),
            Print(format!(" {}", desc))
        )?;
    }

    Ok(())
}

fn render_message(
    stdout: &mut io::Stdout,
    app: &App,
    msg: &str,
    term_width: u16,
) -> io::Result<()> {
    let theme = &app.theme;
    let padded = format!("  {}  ", msg);
    let x = term_width.saturating_sub(padded.len() as u16) / 2;

    execute!(
        stdout,
        MoveTo(x, 0),
        SetForegroundColor(theme.fg),
        SetBackgroundColor(theme.selected_bg),
        Print(&padded)
    )?;

    Ok(())
}

fn render_menu(
    stdout: &mut io::Stdout,
    app: &App,
    term_width: u16,
    term_height: u16,
) -> io::Result<()> {
    let theme = &app.theme;

    // Calculate menu size based on content
    let is_difficulty_menu = matches!(app.menu, MenuState::NewGame | MenuState::Difficulty);
    let num_options = if is_difficulty_menu {
        if app.stats.secrets_unlocked() {
            8
        } else {
            6
        }
    } else {
        match app.menu {
            MenuState::Theme => 3,
            MenuState::Confirm => 2,
            _ => 0,
        }
    };

    let menu_width: u16 = 30;
    let menu_height: u16 = (num_options + 5) as u16; // title + options + padding
    let x = (term_width.saturating_sub(menu_width)) / 2;
    let y = (term_height.saturating_sub(menu_height)) / 2;

    let bg = Color::Rgb {
        r: 30,
        g: 30,
        b: 40,
    };

    // Background
    for row in 0..menu_height {
        execute!(
            stdout,
            MoveTo(x, y + row),
            SetBackgroundColor(bg),
            Print(" ".repeat(menu_width as usize))
        )?;
    }

    // Border
    execute!(
        stdout,
        SetForegroundColor(theme.border),
        SetBackgroundColor(bg)
    )?;
    execute!(
        stdout,
        MoveTo(x, y),
        Print("┌"),
        Print("─".repeat(menu_width as usize - 2)),
        Print("┐")
    )?;
    for row in 1..menu_height - 1 {
        execute!(stdout, MoveTo(x, y + row), Print("│"))?;
        execute!(stdout, MoveTo(x + menu_width - 1, y + row), Print("│"))?;
    }
    execute!(
        stdout,
        MoveTo(x, y + menu_height - 1),
        Print("└"),
        Print("─".repeat(menu_width as usize - 2)),
        Print("┘")
    )?;

    // Title
    let title = match app.menu {
        MenuState::NewGame | MenuState::Difficulty => "Select Difficulty",
        MenuState::Theme => "Select Theme",
        MenuState::Confirm => "Confirm",
        MenuState::None => "",
    };
    let title_x = x + (menu_width.saturating_sub(title.len() as u16)) / 2;
    execute!(
        stdout,
        MoveTo(title_x, y + 1),
        SetForegroundColor(theme.fg),
        SetBackgroundColor(bg),
        Print(title)
    )?;

    // Options
    if is_difficulty_menu {
        let difficulties: Vec<(&str, Color, bool)> = if app.stats.secrets_unlocked() {
            vec![
                ("Beginner", Color::Cyan, true),
                ("Easy", Color::Green, true),
                ("Medium", Color::Yellow, true),
                (
                    "Intermediate",
                    Color::Rgb {
                        r: 255,
                        g: 200,
                        b: 100,
                    },
                    true,
                ),
                (
                    "Hard",
                    Color::Rgb {
                        r: 255,
                        g: 165,
                        b: 0,
                    },
                    true,
                ),
                ("Expert", Color::Red, true),
                ("★ Master", Color::Magenta, true),
                (
                    "★ Extreme",
                    Color::Rgb {
                        r: 255,
                        g: 50,
                        b: 255,
                    },
                    true,
                ),
            ]
        } else {
            vec![
                ("Beginner", Color::Cyan, true),
                ("Easy", Color::Green, true),
                ("Medium", Color::Yellow, true),
                (
                    "Intermediate",
                    Color::Rgb {
                        r: 255,
                        g: 200,
                        b: 100,
                    },
                    true,
                ),
                (
                    "Hard",
                    Color::Rgb {
                        r: 255,
                        g: 165,
                        b: 0,
                    },
                    true,
                ),
                ("Expert", Color::Red, true),
            ]
        };

        for (i, (name, color, _)) in difficulties.iter().enumerate() {
            let selected = i == app.menu_selection;
            let (fg, item_bg) = if selected {
                (Color::Black, *color)
            } else {
                (*color, bg)
            };

            execute!(
                stdout,
                MoveTo(x + 2, y + 3 + i as u16),
                SetForegroundColor(fg),
                SetBackgroundColor(item_bg),
                Print(format!(" {:^24} ", name))
            )?;
        }

        // Show unlock hint if not unlocked
        if !app.stats.secrets_unlocked() {
            let (wins, needed) = app.stats.expert_wins_progress();
            let hint = format!("🔒 {}/{} Expert wins", wins, needed);
            execute!(
                stdout,
                MoveTo(x + 2, y + 3 + difficulties.len() as u16),
                SetForegroundColor(Color::DarkGrey),
                SetBackgroundColor(bg),
                Print(format!(" {:^24} ", hint))
            )?;
        }
    } else {
        let options: &[&str] = match app.menu {
            MenuState::Theme => &["Dark", "Light", "High Contrast"],
            MenuState::Confirm => &["Yes", "No"],
            _ => &[],
        };

        for (i, option) in options.iter().enumerate() {
            let selected = i == app.menu_selection;
            let (fg, item_bg) = if selected {
                (Color::Black, theme.key)
            } else {
                (theme.fg, bg)
            };

            execute!(
                stdout,
                MoveTo(x + 2, y + 3 + i as u16),
                SetForegroundColor(fg),
                SetBackgroundColor(item_bg),
                Print(format!(" {:^24} ", option))
            )?;
        }
    }

    Ok(())
}

fn render_hint(
    stdout: &mut io::Stdout,
    app: &App,
    hint: &str,
    term_width: u16,
    term_height: u16,
) -> io::Result<()> {
    let theme = &app.theme;

    let max_width = 45;
    let wrapped = wrap_text(hint, max_width);

    let box_width = (max_width + 4) as u16;
    let box_height = (wrapped.len() + 4) as u16;
    let x = term_width.saturating_sub(box_width) / 2;
    let y = term_height.saturating_sub(box_height) / 2;

    let bg = Color::Rgb {
        r: 25,
        g: 45,
        b: 25,
    };

    // Background
    for row in 0..box_height {
        execute!(
            stdout,
            MoveTo(x, y + row),
            SetBackgroundColor(bg),
            Print(" ".repeat(box_width as usize))
        )?;
    }

    // Title
    execute!(
        stdout,
        MoveTo(x + 2, y + 1),
        SetForegroundColor(theme.success),
        SetBackgroundColor(bg),
        Print("💡 Hint")
    )?;

    // Text
    for (i, line) in wrapped.iter().enumerate() {
        execute!(
            stdout,
            MoveTo(x + 2, y + 3 + i as u16),
            SetForegroundColor(theme.fg),
            SetBackgroundColor(bg),
            Print(line)
        )?;
    }

    Ok(())
}

// Win/Lose screens

fn render_win_screen(
    stdout: &mut io::Stdout,
    app: &mut App,
    term_width: u16,
    term_height: u16,
) -> io::Result<()> {
    app.win_screen.resize(term_width, term_height);

    // Consistent dark background base
    let bg_base = Color::Rgb { r: 8, g: 12, b: 20 };

    // Background - draw entire screen
    for y in 0..term_height {
        for x in 0..term_width {
            let (ch, color) = app.win_screen.background.render_at(
                x as usize,
                y as usize,
                term_width as usize,
                term_height as usize,
                app.win_screen.frame_count() as f32,
            );
            execute!(
                stdout,
                MoveTo(x, y),
                SetForegroundColor(color),
                SetBackgroundColor(bg_base),
                Print(ch)
            )?;
        }
    }

    // Particles - draw on top of background
    for particle in app.win_screen.particles() {
        if particle.is_visible(term_width, term_height) {
            execute!(
                stdout,
                MoveTo(particle.x as u16, particle.y as u16),
                SetForegroundColor(particle.color),
                SetBackgroundColor(bg_base),
                Print(particle.char)
            )?;
        }
    }

    // Banner
    let banner = app.win_screen.current_banner();
    let lines: Vec<&str> = banner.lines().filter(|l| !l.is_empty()).collect();
    let banner_width = lines.iter().map(|l| l.len()).max().unwrap_or(40) as u16;
    let banner_x = term_width.saturating_sub(banner_width) / 2;
    let banner_y = 3;

    for (i, line) in lines.iter().enumerate() {
        let hue = (app.win_screen.rainbow_offset() + i as f32 * 0.1) % 1.0;
        execute!(
            stdout,
            MoveTo(banner_x, banner_y + i as u16),
            SetForegroundColor(hue_to_rgb(hue)),
            SetBackgroundColor(bg_base),
            Print(line)
        )?;
    }

    // Message
    let msg = app.win_screen.current_message();
    let msg_x = term_width.saturating_sub(msg.len() as u16) / 2;
    let msg_y = banner_y + lines.len() as u16 + 2;
    let hue = (app.win_screen.rainbow_offset() * 2.0) % 1.0;

    execute!(
        stdout,
        MoveTo(msg_x, msg_y),
        SetForegroundColor(hue_to_rgb(hue)),
        SetBackgroundColor(bg_base),
        Print(msg)
    )?;

    // Stats box
    let stats = format!(
        "Time: {} | Hints: {} | Difficulty: {}",
        app.game.elapsed_string(),
        app.game.hints_used(),
        app.game.difficulty()
    );
    let stats_x = term_width.saturating_sub(stats.len() as u16 + 2) / 2;
    execute!(
        stdout,
        MoveTo(stats_x, msg_y + 3),
        SetForegroundColor(Color::White),
        SetBackgroundColor(Color::Rgb {
            r: 30,
            g: 50,
            b: 30
        }),
        Print(format!(" {} ", stats))
    )?;

    // Instructions
    let instr = "Press 'n' for new game or 'q' to quit";
    let instr_x = term_width.saturating_sub(instr.len() as u16) / 2;
    execute!(
        stdout,
        MoveTo(instr_x, msg_y + 5),
        SetForegroundColor(Color::Yellow),
        SetBackgroundColor(bg_base),
        Print(instr)
    )?;

    Ok(())
}

fn render_lose_screen(
    stdout: &mut io::Stdout,
    app: &mut App,
    term_width: u16,
    term_height: u16,
) -> io::Result<()> {
    app.lose_screen.resize(term_width, term_height);

    // Consistent dark background base
    let bg_base = Color::Rgb {
        r: 15,
        g: 10,
        b: 12,
    };

    // Background - draw entire screen
    for y in 0..term_height {
        for x in 0..term_width {
            let (ch, color) = app.lose_screen.background.render_at(
                x as usize,
                y as usize,
                term_width as usize,
                term_height as usize,
                app.lose_screen.frame_count() as f32,
            );
            execute!(
                stdout,
                MoveTo(x, y),
                SetForegroundColor(color),
                SetBackgroundColor(bg_base),
                Print(ch)
            )?;
        }
    }

    // Particles - draw on top of background
    for particle in app.lose_screen.particles() {
        if particle.is_visible(term_width, term_height) {
            execute!(
                stdout,
                MoveTo(particle.x as u16, particle.y as u16),
                SetForegroundColor(particle.color),
                SetBackgroundColor(bg_base),
                Print(particle.char)
            )?;
        }
    }

    // Banner
    let banner = app.lose_screen.current_banner();
    let lines: Vec<&str> = banner.lines().filter(|l| !l.is_empty()).collect();
    let banner_width = lines.iter().map(|l| l.len()).max().unwrap_or(40) as u16;
    let banner_x = term_width.saturating_sub(banner_width) / 2;
    let banner_y = 3;

    for (i, line) in lines.iter().enumerate() {
        let intensity = 150u8.saturating_sub((i * 10).min(100) as u8);
        execute!(
            stdout,
            MoveTo(banner_x, banner_y + i as u16),
            SetForegroundColor(Color::Rgb {
                r: intensity,
                g: 30,
                b: 30
            }),
            SetBackgroundColor(bg_base),
            Print(line)
        )?;
    }

    // Message with pulse
    let msg = app.lose_screen.current_message();
    let msg_x = term_width.saturating_sub(msg.len() as u16) / 2;
    let msg_y = banner_y + lines.len() as u16 + 2;
    let pulse = ((app.lose_screen.frame_count() as f32 * 0.1).sin() * 0.3 + 0.7) * 255.0;

    execute!(
        stdout,
        MoveTo(msg_x, msg_y),
        SetForegroundColor(Color::Rgb {
            r: pulse as u8,
            g: 50,
            b: 50
        }),
        SetBackgroundColor(bg_base),
        Print(msg)
    )?;

    // Stats box
    let stats = format!(
        "Mistakes: {}/{} | Time: {} | Difficulty: {}",
        app.game.mistakes(),
        MAX_MISTAKES,
        app.game.elapsed_string(),
        app.game.difficulty()
    );
    let stats_x = term_width.saturating_sub(stats.len() as u16 + 2) / 2;
    execute!(
        stdout,
        MoveTo(stats_x, msg_y + 3),
        SetForegroundColor(Color::Grey),
        SetBackgroundColor(Color::Rgb {
            r: 30,
            g: 20,
            b: 20
        }),
        Print(format!(" {} ", stats))
    )?;

    // Instructions
    let instr = "Press 'n' for new game or 'q' to quit";
    let instr_x = term_width.saturating_sub(instr.len() as u16) / 2;
    execute!(
        stdout,
        MoveTo(instr_x, msg_y + 5),
        SetForegroundColor(Color::DarkYellow),
        SetBackgroundColor(bg_base),
        Print(instr)
    )?;

    Ok(())
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.len() + word.len() + 1 > max_width && !current.is_empty() {
            lines.push(current);
            current = String::new();
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

// Stats, Leaderboard, History screens

fn render_stats_screen(
    stdout: &mut io::Stdout,
    app: &App,
    term_width: u16,
    term_height: u16,
) -> io::Result<()> {
    let theme = &app.theme;
    let stats = &app.stats;
    let player = &stats.player;

    execute!(stdout, SetBackgroundColor(theme.bg))?;

    // Title
    let title = "═══ STATISTICS ═══";
    let title_x = term_width.saturating_sub(title.len() as u16) / 2;
    execute!(
        stdout,
        MoveTo(title_x, 1),
        SetForegroundColor(theme.key),
        Print(title)
    )?;

    let start_y = 3;
    let col1_x = 4u16;
    let col2_x = term_width / 2;

    // Overall stats (left column)
    execute!(
        stdout,
        MoveTo(col1_x, start_y),
        SetForegroundColor(theme.fg),
        Print(format!("Player: {}", player.player_name))
    )?;

    execute!(
        stdout,
        MoveTo(col1_x, start_y + 2),
        SetForegroundColor(theme.info),
        Print(format!("Total Games: {}", player.total_games))
    )?;
    execute!(
        stdout,
        MoveTo(col1_x, start_y + 3),
        SetForegroundColor(theme.success),
        Print(format!("Wins: {}", player.total_wins))
    )?;
    execute!(
        stdout,
        MoveTo(col1_x, start_y + 4),
        SetForegroundColor(theme.error),
        Print(format!("Losses: {}", player.total_losses))
    )?;
    execute!(
        stdout,
        MoveTo(col1_x, start_y + 5),
        SetForegroundColor(theme.border),
        Print(format!("Abandoned: {}", player.total_abandoned))
    )?;
    execute!(
        stdout,
        MoveTo(col1_x, start_y + 7),
        SetForegroundColor(theme.fg),
        Print(format!("Win Rate: {:.1}%", player.overall_win_rate()))
    )?;

    // Streak info
    let streak_color = if player.current_streak > 0 {
        theme.success
    } else if player.current_streak < 0 {
        theme.error
    } else {
        theme.info
    };
    execute!(
        stdout,
        MoveTo(col1_x, start_y + 8),
        SetForegroundColor(streak_color),
        Print(format!("Current Streak: {}", player.current_streak.abs())),
        SetForegroundColor(theme.info),
        Print(if player.current_streak > 0 {
            " wins"
        } else if player.current_streak < 0 {
            " losses"
        } else {
            ""
        })
    )?;
    execute!(
        stdout,
        MoveTo(col1_x, start_y + 9),
        SetForegroundColor(theme.key),
        Print(format!("Best Streak: {} wins", player.best_streak))
    )?;

    // Per-difficulty stats (right column)
    execute!(
        stdout,
        MoveTo(col2_x, start_y),
        SetForegroundColor(theme.fg),
        Print("By Difficulty:")
    )?;

    let difficulties = [
        sudoku_core::Difficulty::Easy,
        sudoku_core::Difficulty::Medium,
        sudoku_core::Difficulty::Hard,
        sudoku_core::Difficulty::Expert,
    ];

    for (i, diff) in difficulties.iter().enumerate() {
        let ds = player.get_difficulty_stats(*diff);
        let y = start_y + 2 + (i as u16 * 4);

        let diff_color = match diff {
            sudoku_core::Difficulty::Beginner => Color::Cyan,
            sudoku_core::Difficulty::Easy => Color::Green,
            sudoku_core::Difficulty::Medium => Color::Yellow,
            sudoku_core::Difficulty::Intermediate => Color::Rgb {
                r: 255,
                g: 200,
                b: 100,
            },
            sudoku_core::Difficulty::Hard => Color::Rgb {
                r: 255,
                g: 165,
                b: 0,
            },
            sudoku_core::Difficulty::Expert => Color::Red,
            sudoku_core::Difficulty::Master => Color::Magenta,
            sudoku_core::Difficulty::Extreme => Color::Rgb {
                r: 255,
                g: 50,
                b: 255,
            },
        };

        execute!(
            stdout,
            MoveTo(col2_x, y),
            SetForegroundColor(diff_color),
            Print(format!("{:?}", diff))
        )?;
        execute!(
            stdout,
            MoveTo(col2_x + 2, y + 1),
            SetForegroundColor(theme.info),
            Print(format!(
                "Games: {} | Wins: {} ({:.0}%)",
                ds.total_games,
                ds.wins,
                ds.win_rate()
            ))
        )?;

        let best_str = ds
            .best_time_secs
            .map(format_time)
            .unwrap_or_else(|| "--:--".to_string());
        let avg_str = ds
            .avg_time_secs()
            .map(format_time)
            .unwrap_or_else(|| "--:--".to_string());
        execute!(
            stdout,
            MoveTo(col2_x + 2, y + 2),
            SetForegroundColor(theme.info),
            Print(format!("Best: {} | Avg: {}", best_str, avg_str))
        )?;
    }

    // Navigation help
    let nav_y = term_height.saturating_sub(3);
    execute!(
        stdout,
        MoveTo(col1_x, nav_y),
        SetForegroundColor(theme.border),
        Print("────────────────────────────────────────────────────────────")
    )?;
    execute!(
        stdout,
        MoveTo(col1_x, nav_y + 1),
        SetForegroundColor(theme.key),
        Print("b"),
        SetForegroundColor(theme.info),
        Print(" Leaderboard  "),
        SetForegroundColor(theme.key),
        Print("H"),
        SetForegroundColor(theme.info),
        Print(" History  "),
        SetForegroundColor(theme.key),
        Print("Esc"),
        SetForegroundColor(theme.info),
        Print(" Back to game")
    )?;

    Ok(())
}

fn render_leaderboard_screen(
    stdout: &mut io::Stdout,
    app: &App,
    term_width: u16,
    term_height: u16,
) -> io::Result<()> {
    let theme = &app.theme;
    let stats = &app.stats;

    execute!(stdout, SetBackgroundColor(theme.bg))?;

    // Title
    let title = "═══ LEADERBOARD ═══";
    let title_x = term_width.saturating_sub(title.len() as u16) / 2;
    execute!(
        stdout,
        MoveTo(title_x, 1),
        SetForegroundColor(theme.key),
        Print(title)
    )?;

    // Difficulty filter
    let diff_y = 3;
    let difficulties: Vec<&str> = if app.stats.secrets_unlocked() {
        vec![
            "Beginner", "Easy", "Medium", "Inter", "Hard", "Expert", "Master", "Extreme",
        ]
    } else {
        vec!["Beginner", "Easy", "Medium", "Inter", "Hard", "Expert"]
    };
    let current_idx = match app.leaderboard_difficulty {
        sudoku_core::Difficulty::Beginner => 0,
        sudoku_core::Difficulty::Easy => 1,
        sudoku_core::Difficulty::Medium => 2,
        sudoku_core::Difficulty::Intermediate => 3,
        sudoku_core::Difficulty::Hard => 4,
        sudoku_core::Difficulty::Expert => 5,
        sudoku_core::Difficulty::Master => 6,
        sudoku_core::Difficulty::Extreme => 7,
    };

    execute!(
        stdout,
        MoveTo(4, diff_y),
        SetForegroundColor(theme.info),
        Print("◀ ")
    )?;

    for (i, name) in difficulties.iter().enumerate() {
        let color = if i == current_idx {
            theme.key
        } else {
            theme.border
        };
        execute!(
            stdout,
            SetForegroundColor(color),
            Print(format!(" {} ", name))
        )?;
    }
    execute!(stdout, SetForegroundColor(theme.info), Print(" ▶"))?;

    // Header
    let header_y = diff_y + 2;
    execute!(
        stdout,
        MoveTo(4, header_y),
        SetForegroundColor(theme.fg),
        Print(format!(
            "{:>4} {:>12} {:>8} {:>6} {:>8} {:>10}",
            "Rank", "Player", "Score", "Time", "Hints", "Verified"
        ))
    )?;
    execute!(
        stdout,
        MoveTo(4, header_y + 1),
        SetForegroundColor(theme.border),
        Print("─".repeat(60))
    )?;

    // Leaderboard entries
    let entries = stats.leaderboard_by_difficulty(app.leaderboard_difficulty);
    let max_entries = (term_height.saturating_sub(header_y + 5)) as usize;

    for (i, entry) in entries.iter().take(max_entries).enumerate() {
        let y = header_y + 2 + i as u16;
        let rank_color = match i {
            0 => Color::Yellow, // Gold
            1 => Color::Grey,   // Silver
            2 => Color::Rgb {
                r: 205,
                g: 127,
                b: 50,
            }, // Bronze
            _ => theme.info,
        };

        execute!(
            stdout,
            MoveTo(4, y),
            SetForegroundColor(rank_color),
            Print(format!("{:>4}", i + 1)),
            SetForegroundColor(theme.fg),
            Print(format!(
                " {:>12}",
                &entry.player_name[..entry.player_name.len().min(12)]
            )),
            SetForegroundColor(theme.key),
            Print(format!(" {:>8}", entry.score)),
            SetForegroundColor(theme.info),
            Print(format!(" {:>8}", format_time(entry.time_secs))),
            Print(format!(" {:>6}", entry.hints_used)),
            SetForegroundColor(theme.success),
            Print(format!(" {:>10}", "✓"))
        )?;
    }

    if entries.is_empty() {
        execute!(
            stdout,
            MoveTo(4, header_y + 3),
            SetForegroundColor(theme.border),
            Print("No entries yet. Win some games!")
        )?;
    }

    // Navigation help
    let nav_y = term_height.saturating_sub(3);
    execute!(
        stdout,
        MoveTo(4, nav_y),
        SetForegroundColor(theme.border),
        Print("────────────────────────────────────────────────────────────")
    )?;
    execute!(
        stdout,
        MoveTo(4, nav_y + 1),
        SetForegroundColor(theme.key),
        Print("←/→"),
        SetForegroundColor(theme.info),
        Print(" Change difficulty  "),
        SetForegroundColor(theme.key),
        Print("i"),
        SetForegroundColor(theme.info),
        Print(" Stats  "),
        SetForegroundColor(theme.key),
        Print("Esc"),
        SetForegroundColor(theme.info),
        Print(" Back")
    )?;

    Ok(())
}

fn render_history_screen(
    stdout: &mut io::Stdout,
    app: &App,
    term_width: u16,
    term_height: u16,
) -> io::Result<()> {
    let theme = &app.theme;
    let stats = &app.stats;

    execute!(stdout, SetBackgroundColor(theme.bg))?;

    // Title
    let title = "═══ GAME HISTORY ═══";
    let title_x = term_width.saturating_sub(title.len() as u16) / 2;
    execute!(
        stdout,
        MoveTo(title_x, 1),
        SetForegroundColor(theme.key),
        Print(title)
    )?;

    // Subtitle
    execute!(
        stdout,
        MoveTo(4, 3),
        SetForegroundColor(theme.info),
        Print("Select a game and press Enter to replay")
    )?;

    // Header
    let header_y = 5;
    execute!(
        stdout,
        MoveTo(4, header_y),
        SetForegroundColor(theme.fg),
        Print(format!(
            "{:>4} {:>8} {:>8} {:>8} {:>6} {:>8} {:>10}",
            "#", "Result", "Diff", "Time", "Hints", "Errors", "Verified"
        ))
    )?;
    execute!(
        stdout,
        MoveTo(4, header_y + 1),
        SetForegroundColor(theme.border),
        Print("─".repeat(65))
    )?;

    // History entries
    let visible_rows = (term_height.saturating_sub(header_y + 6)) as usize;
    let history = stats.recent_games(100);

    for (i, record) in history
        .iter()
        .skip(app.history_scroll)
        .take(visible_rows)
        .enumerate()
    {
        let y = header_y + 2 + i as u16;
        let is_selected = i + app.history_scroll == app.history_scroll;
        let actual_idx = i + app.history_scroll;

        let bg = if is_selected && actual_idx == app.history_scroll {
            theme.selected_bg
        } else {
            theme.bg
        };

        execute!(stdout, SetBackgroundColor(bg))?;

        let result_str = match record.result {
            GameResult::Win => "WIN",
            GameResult::Loss => "LOSS",
            GameResult::Abandoned => "QUIT",
        };
        let result_color = match record.result {
            GameResult::Win => theme.success,
            GameResult::Loss => theme.error,
            GameResult::Abandoned => theme.border,
        };

        let diff_str = format!("{:?}", record.difficulty);
        let verified_str = if record.verified { "✓" } else { "✗" };
        let verified_color = if record.verified {
            theme.success
        } else {
            theme.error
        };

        // Highlight selected row
        let prefix = if actual_idx == app.history_scroll {
            "▶"
        } else {
            " "
        };

        execute!(
            stdout,
            MoveTo(2, y),
            SetForegroundColor(theme.key),
            Print(prefix),
            MoveTo(4, y),
            SetForegroundColor(theme.info),
            Print(format!("{:>4}", record.id)),
            SetForegroundColor(result_color),
            Print(format!(" {:>8}", result_str)),
            SetForegroundColor(theme.info),
            Print(format!(" {:>8}", diff_str)),
            Print(format!(" {:>8}", format_time(record.time_secs))),
            Print(format!(" {:>6}", record.hints_used)),
            Print(format!(" {:>8}", record.mistakes)),
            SetForegroundColor(verified_color),
            Print(format!(" {:>10}", verified_str))
        )?;

        execute!(stdout, SetBackgroundColor(theme.bg))?;
    }

    if history.is_empty() {
        execute!(
            stdout,
            MoveTo(4, header_y + 3),
            SetForegroundColor(theme.border),
            Print("No games played yet!")
        )?;
    }

    // Scroll indicator
    if history.len() > visible_rows {
        let scroll_y = header_y + 2;
        let scroll_height = visible_rows as u16;
        let scroll_pos =
            (app.history_scroll as f32 / history.len() as f32 * scroll_height as f32) as u16;

        for i in 0..scroll_height {
            let ch = if i == scroll_pos { '█' } else { '░' };
            execute!(
                stdout,
                MoveTo(term_width - 3, scroll_y + i),
                SetForegroundColor(theme.border),
                Print(ch)
            )?;
        }
    }

    // Navigation help
    let nav_y = term_height.saturating_sub(3);
    execute!(
        stdout,
        MoveTo(4, nav_y),
        SetForegroundColor(theme.border),
        Print("────────────────────────────────────────────────────────────────")
    )?;
    execute!(
        stdout,
        MoveTo(4, nav_y + 1),
        SetForegroundColor(theme.key),
        Print("↑/↓"),
        SetForegroundColor(theme.info),
        Print(" Select  "),
        SetForegroundColor(theme.key),
        Print("Enter"),
        SetForegroundColor(theme.info),
        Print(" Replay  "),
        SetForegroundColor(theme.key),
        Print("i"),
        SetForegroundColor(theme.info),
        Print(" Stats  "),
        SetForegroundColor(theme.key),
        Print("Esc"),
        SetForegroundColor(theme.info),
        Print(" Back")
    )?;

    Ok(())
}
