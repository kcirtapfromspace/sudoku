//! Canvas rendering for terminal-like Sudoku UI

use crate::game::{GameState, InputMode, ScreenState, MAX_MISTAKES};
use crate::theme::Theme;
use sudoku_core::Position;
use web_sys::CanvasRenderingContext2d;

// Box-drawing characters for terminal feel (reserved for future text-based rendering)
#[allow(dead_code)]
mod box_chars {
    pub const HORIZONTAL: &str = "─";
    pub const VERTICAL: &str = "│";
    pub const TOP_LEFT: &str = "┌";
    pub const TOP_RIGHT: &str = "┐";
    pub const BOTTOM_LEFT: &str = "└";
    pub const BOTTOM_RIGHT: &str = "┘";
    pub const CROSS: &str = "┼";
    pub const T_DOWN: &str = "┬";
    pub const T_UP: &str = "┴";
    pub const T_RIGHT: &str = "├";
    pub const T_LEFT: &str = "┤";
    pub const H_THICK: &str = "━";
    pub const V_THICK: &str = "┃";
    pub const TL_THICK: &str = "┏";
    pub const TR_THICK: &str = "┓";
    pub const BL_THICK: &str = "┗";
    pub const BR_THICK: &str = "┛";
    pub const CROSS_THICK: &str = "╋";
    pub const T_DOWN_THICK: &str = "┳";
    pub const T_UP_THICK: &str = "┻";
    pub const T_RIGHT_THICK: &str = "┣";
    pub const T_LEFT_THICK: &str = "┫";
}

/// Render the complete game to canvas
pub fn render_game(
    ctx: &CanvasRenderingContext2d,
    state: &GameState,
    theme: &Theme,
    width: u32,
    height: u32,
    cell_size: f64,
    font_size: f64,
) {
    // Clear background
    ctx.set_fill_style_str(&theme.background.as_css());
    ctx.fill_rect(0.0, 0.0, width as f64, height as f64);

    // Calculate grid position (centered with room for info panel)
    let grid_width = cell_size * 9.0 + 4.0; // 9 cells + borders
    let grid_height = cell_size * 9.0 + 4.0;
    let grid_x = 40.0;
    let grid_y = (height as f64 - grid_height) / 2.0;

    match state.screen() {
        ScreenState::Playing | ScreenState::Paused => {
            render_grid(ctx, state, theme, grid_x, grid_y, cell_size, font_size);
            render_info_panel(
                ctx,
                state,
                theme,
                grid_x + grid_width + 30.0,
                grid_y,
                font_size,
            );

            if state.screen() == ScreenState::Paused {
                render_pause_overlay(ctx, theme, width, height, font_size);
            }
        }
        ScreenState::Win => {
            render_grid(ctx, state, theme, grid_x, grid_y, cell_size, font_size);
            render_win_screen(ctx, state, theme, width, height, font_size);
        }
        ScreenState::Lose => {
            render_grid(ctx, state, theme, grid_x, grid_y, cell_size, font_size);
            render_lose_screen(ctx, state, theme, width, height, font_size);
        }
        ScreenState::Menu => {
            render_grid(ctx, state, theme, grid_x, grid_y, cell_size, font_size);
            render_menu(ctx, theme, width, height, font_size);
        }
    }

    // Render message if present
    if let Some(msg) = state.message() {
        render_message(ctx, theme, msg, width, height, font_size);
    }
}

/// Render the Sudoku grid
fn render_grid(
    ctx: &CanvasRenderingContext2d,
    state: &GameState,
    theme: &Theme,
    x: f64,
    y: f64,
    cell_size: f64,
    font_size: f64,
) {
    let cursor = state.cursor();
    let completed = state.completed_numbers();

    // Set font for numbers
    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', 'Fira Code', 'Consolas', monospace",
        font_size
    ));
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");

    // Draw cells
    for row in 0..9 {
        for col in 0..9 {
            let pos = Position::new(row, col);
            let cell = state.grid().cell(pos);
            let cell_x = x + col as f64 * cell_size;
            let cell_y = y + row as f64 * cell_size;

            // Determine cell background
            let bg_color = if pos == cursor {
                &theme.cursor_bg
            } else if state.has_same_value(pos) && state.grid().get(cursor).is_some() {
                &theme.same_value_bg
            } else if state.is_highlighted(pos) {
                &theme.highlight_bg
            } else {
                &theme.cell_bg
            };

            // Draw cell background
            ctx.set_fill_style_str(&bg_color.as_css());
            ctx.fill_rect(cell_x, cell_y, cell_size, cell_size);

            // Highlight naked singles if valid cells mode is on
            if state.show_valid_cells() && state.is_naked_single(pos) {
                ctx.set_stroke_style_str(&theme.win_color.as_css_alpha(0.6));
                ctx.set_line_width(2.0);
                ctx.stroke_rect(cell_x + 2.0, cell_y + 2.0, cell_size - 4.0, cell_size - 4.0);
            }

            // Draw cell content
            if let Some(value) = cell.value() {
                // Check for conflict
                let has_conflict = state.has_conflict(pos);

                // Determine text color
                let text_color = if has_conflict {
                    &theme.error_text
                } else if cell.is_given() {
                    &theme.given_text
                } else {
                    &theme.player_text
                };

                ctx.set_fill_style_str(&text_color.as_css());
                let _ = ctx.fill_text(
                    &value.to_string(),
                    cell_x + cell_size / 2.0,
                    cell_y + cell_size / 2.0,
                );
            } else {
                // Draw candidates (pencil marks) or ghost hints
                let candidates = cell.candidates();
                let ghost_candidates = if state.show_ghost_hints() && candidates.is_empty() {
                    state.get_ghost_candidates(pos)
                } else {
                    Vec::new()
                };

                let small_font = font_size * 0.45;
                ctx.set_font(&format!(
                    "bold {}px 'JetBrains Mono', monospace",
                    small_font
                ));

                // Draw user's candidates
                if !candidates.is_empty() {
                    ctx.set_fill_style_str(&theme.candidate_text.as_css());
                    for v in candidates.iter() {
                        let (dx, dy) = candidate_offset(v);
                        let cx = cell_x + cell_size * dx;
                        let cy = cell_y + cell_size * dy;
                        let _ = ctx.fill_text(&v.to_string(), cx, cy);
                    }
                }

                // Draw ghost candidates (faded)
                if !ghost_candidates.is_empty() {
                    ctx.set_fill_style_str(&theme.candidate_text.as_css_alpha(0.4));
                    for v in ghost_candidates.iter() {
                        let (dx, dy) = candidate_offset(*v);
                        let cx = cell_x + cell_size * dx;
                        let cy = cell_y + cell_size * dy;
                        let _ = ctx.fill_text(&v.to_string(), cx, cy);
                    }
                }

                // Reset font
                ctx.set_font(&format!("{}px 'JetBrains Mono', monospace", font_size));
            }
        }
    }

    // Draw grid lines
    ctx.set_stroke_style_str(&theme.grid_lines.as_css());
    ctx.set_line_width(1.0);

    for i in 0..=9 {
        let offset = i as f64 * cell_size;

        // Vertical lines
        ctx.begin_path();
        ctx.move_to(x + offset, y);
        ctx.line_to(x + offset, y + 9.0 * cell_size);
        ctx.stroke();

        // Horizontal lines
        ctx.begin_path();
        ctx.move_to(x, y + offset);
        ctx.line_to(x + 9.0 * cell_size, y + offset);
        ctx.stroke();
    }

    // Draw thick box borders
    ctx.set_stroke_style_str(&theme.box_border.as_css());
    ctx.set_line_width(3.0);

    for i in 0..=3 {
        let offset = i as f64 * cell_size * 3.0;

        // Vertical thick lines
        ctx.begin_path();
        ctx.move_to(x + offset, y);
        ctx.line_to(x + offset, y + 9.0 * cell_size);
        ctx.stroke();

        // Horizontal thick lines
        ctx.begin_path();
        ctx.move_to(x, y + offset);
        ctx.line_to(x + 9.0 * cell_size, y + offset);
        ctx.stroke();
    }

    // Draw cursor outline
    ctx.set_stroke_style_str(&theme.cursor_bg.as_css());
    ctx.set_line_width(3.0);
    let cursor_x = x + cursor.col as f64 * cell_size;
    let cursor_y = y + cursor.row as f64 * cell_size;
    ctx.stroke_rect(cursor_x, cursor_y, cell_size, cell_size);

    // Draw number completion indicator at bottom
    let indicator_y = y + 9.0 * cell_size + 20.0;
    ctx.set_font(&format!("{}px monospace", font_size * 0.6));
    ctx.set_text_align("center");

    for (i, &is_completed) in completed.iter().enumerate() {
        let num = (i + 1) as u8;
        let indicator_x = x + (i as f64 + 0.5) * cell_size;

        if is_completed {
            ctx.set_fill_style_str(&theme.completed_bg.as_css());
            ctx.fill_rect(indicator_x - 10.0, indicator_y - 10.0, 20.0, 20.0);
            ctx.set_fill_style_str(&theme.given_text.as_css());
        } else {
            ctx.set_fill_style_str(&theme.candidate_text.as_css());
        }

        let _ = ctx.fill_text(&num.to_string(), indicator_x, indicator_y);
    }
}

/// Get offset for candidate number in 3x3 grid within cell
fn candidate_offset(value: u8) -> (f64, f64) {
    let row = (value - 1) / 3;
    let col = (value - 1) % 3;
    let dx = 0.2 + col as f64 * 0.3;
    let dy = 0.2 + row as f64 * 0.3;
    (dx, dy)
}

/// Render the info panel
fn render_info_panel(
    ctx: &CanvasRenderingContext2d,
    state: &GameState,
    theme: &Theme,
    x: f64,
    y: f64,
    font_size: f64,
) {
    let info_font = font_size * 0.65;
    let small_font = font_size * 0.5;
    let line_height = font_size * 0.9;
    let small_line = font_size * 0.55;

    ctx.set_text_align("left");
    ctx.set_text_baseline("top");

    let mut cy = y;

    // Game stats section
    ctx.set_font(&format!("bold {}px 'JetBrains Mono', monospace", info_font));
    ctx.set_fill_style_str(&theme.given_text.as_css());
    let _ = ctx.fill_text("Game", x, cy);
    cy += line_height;

    ctx.set_font(&format!("{}px 'JetBrains Mono', monospace", info_font));
    ctx.set_fill_style_str(&theme.info_text.as_css());

    let _ = ctx.fill_text(&format!("Time: {}", state.elapsed_string()), x, cy);
    cy += line_height;

    let _ = ctx.fill_text(&format!("{}", state.difficulty()), x, cy);
    cy += line_height;

    let remaining = MAX_MISTAKES.saturating_sub(state.mistakes());
    let hearts: String = "♥".repeat(remaining) + &"♡".repeat(state.mistakes());
    let _ = ctx.fill_text(
        &format!("{} │ Hints: {}", hearts, state.hints_used()),
        x,
        cy,
    );
    cy += line_height;

    let mode_str = match state.mode() {
        InputMode::Normal => "Normal",
        InputMode::Candidate => "Notes",
    };
    let ghost = if state.show_ghost_hints() { "G" } else { "-" };
    let valid = if state.show_valid_cells() { "V" } else { "-" };
    let _ = ctx.fill_text(&format!("{} │ [{}{}]", mode_str, ghost, valid), x, cy);
    cy += line_height * 1.3;

    // Number completion
    ctx.set_font(&format!("bold {}px 'JetBrains Mono', monospace", info_font));
    ctx.set_fill_style_str(&theme.given_text.as_css());
    let _ = ctx.fill_text("Numbers", x, cy);
    cy += line_height;

    let completed = state.completed_numbers();
    ctx.set_font(&format!("{}px 'JetBrains Mono', monospace", info_font));
    let mut num_line = String::new();
    for (i, &is_completed) in completed.iter().enumerate() {
        if is_completed {
            num_line.push('✓');
        } else {
            num_line.push_str(&format!("{}", i + 1));
        }
        if i < 8 {
            num_line.push(' ');
        }
    }
    ctx.set_fill_style_str(&theme.info_text.as_css());
    let _ = ctx.fill_text(&num_line, x, cy);
    cy += line_height * 1.3;

    // Controls - single column, compact
    ctx.set_font(&format!("bold {}px 'JetBrains Mono', monospace", info_font));
    ctx.set_fill_style_str(&theme.given_text.as_css());
    let _ = ctx.fill_text("Controls", x, cy);
    cy += line_height;

    ctx.set_font(&format!("{}px 'JetBrains Mono', monospace", small_font));
    ctx.set_fill_style_str(&theme.candidate_text.as_css());

    let controls = [
        "↑↓←→ hjkl   Move",
        "wasd        Jump box",
        "1-9         Number",
        "Shift+1-9   Toggle note",
        "0/Del       Clear cell",
        "c           Mode",
        "f           Fill notes",
        "F           Fill ALL notes",
        "x           Clear notes",
        "X           Clear ALL notes",
        "g           Ghost hints",
        "v           Valid cells",
        "? / !       Hint/Apply",
        "u           Undo",
        "p   n       Pause/New",
    ];

    for line in controls {
        let _ = ctx.fill_text(line, x, cy);
        cy += small_line;
    }
}

/// Render pause overlay
fn render_pause_overlay(
    ctx: &CanvasRenderingContext2d,
    theme: &Theme,
    width: u32,
    height: u32,
    font_size: f64,
) {
    // Semi-transparent overlay
    ctx.set_fill_style_str(&theme.background.as_css_alpha(0.9));
    ctx.fill_rect(0.0, 0.0, width as f64, height as f64);

    // Pause text
    ctx.set_font(&format!(
        "bold {}px 'JetBrains Mono', monospace",
        font_size * 2.0
    ));
    ctx.set_fill_style_str(&theme.message_text.as_css());
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text("PAUSED", width as f64 / 2.0, height as f64 / 2.0 - 30.0);

    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', monospace",
        font_size * 0.8
    ));
    ctx.set_fill_style_str(&theme.info_text.as_css());
    let _ = ctx.fill_text(
        "Press P or Space to resume",
        width as f64 / 2.0,
        height as f64 / 2.0 + 30.0,
    );
}

/// Render win screen with animated particles and ASCII banner
fn render_win_screen(
    ctx: &CanvasRenderingContext2d,
    state: &GameState,
    theme: &Theme,
    width: u32,
    height: u32,
    font_size: f64,
) {
    let w = width as f64;
    let h = height as f64;

    if let Some(win_screen) = state.win_screen() {
        // Render animated background
        let frame = win_screen.frame_count() as f32;
        let bg = &win_screen.background;

        // Draw background pattern (sparse sampling for performance)
        let step = 20.0;
        let mut y = 0.0;
        while y < h {
            let mut x = 0.0;
            while x < w {
                let color = bg.color_at(x as f32, y as f32, w as f32, h as f32, frame);
                ctx.set_fill_style_str(&color.as_css_alpha(0.3));
                ctx.fill_rect(x, y, step, step);
                x += step;
            }
            y += step;
        }

        // Render particles
        for particle in win_screen.particles() {
            ctx.set_fill_style_str(&particle.color.as_css());
            ctx.set_font(&format!("{}px 'JetBrains Mono', monospace", particle.size));
            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");
            let alpha = (particle.lifetime.min(1.0) * 255.0) as u8;
            if alpha > 50 {
                let _ = ctx.fill_text(
                    &particle.char.to_string(),
                    particle.x as f64,
                    particle.y as f64,
                );
            }
        }

        // Render ASCII banner with rainbow effect
        let banner = win_screen.current_banner();
        let lines: Vec<&str> = banner.lines().filter(|l| !l.is_empty()).collect();
        let banner_height = lines.len() as f64 * 14.0;
        let start_y = h / 2.0 - banner_height / 2.0 - 80.0;

        ctx.set_font("12px 'JetBrains Mono', monospace");
        ctx.set_text_align("center");
        ctx.set_text_baseline("top");

        for (i, line) in lines.iter().enumerate() {
            // Rainbow color based on line and frame
            let hue = ((i as f32 * 0.1 + win_screen.rainbow_offset()) % 1.0) * 360.0;
            let color = format!("hsl({}, 100%, 60%)", hue);
            ctx.set_fill_style_str(&color);
            let _ = ctx.fill_text(line, w / 2.0, start_y + i as f64 * 14.0);
        }

        // Win message
        ctx.set_font(&format!(
            "bold {}px 'JetBrains Mono', monospace",
            font_size * 1.5
        ));
        let msg_hue = (win_screen.rainbow_offset() * 360.0) % 360.0;
        ctx.set_fill_style_str(&format!("hsl({}, 100%, 70%)", msg_hue));
        ctx.set_text_baseline("middle");
        let _ = ctx.fill_text(win_screen.current_message(), w / 2.0, h / 2.0 + 20.0);
    } else {
        // Fallback: simple overlay
        ctx.set_fill_style_str(&theme.win_color.as_css_alpha(0.15));
        ctx.fill_rect(0.0, 0.0, w, h);
    }

    // Stats
    ctx.set_font(&format!("{}px 'JetBrains Mono', monospace", font_size));
    ctx.set_fill_style_str(&theme.info_text.as_css());
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text(
        &format!(
            "Time: {}  Hints: {}  Mistakes: {}",
            state.elapsed_string(),
            state.hints_used(),
            state.mistakes()
        ),
        w / 2.0,
        h / 2.0 + 70.0,
    );

    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', monospace",
        font_size * 0.7
    ));
    ctx.set_fill_style_str(&theme.info_text.as_css_alpha(0.8));
    let _ = ctx.fill_text(
        "Press N for new game, 1-6 for difficulty",
        w / 2.0,
        h / 2.0 + 110.0,
    );
}

/// Render lose screen with rain/debris particles
fn render_lose_screen(
    ctx: &CanvasRenderingContext2d,
    state: &GameState,
    theme: &Theme,
    width: u32,
    height: u32,
    font_size: f64,
) {
    let w = width as f64;
    let h = height as f64;

    // Dark overlay
    ctx.set_fill_style_str(&theme.lose_color.as_css_alpha(0.2));
    ctx.fill_rect(0.0, 0.0, w, h);

    // Render rain/debris particles
    if let Some(lose_screen) = state.lose_screen() {
        for particle in lose_screen.particles() {
            ctx.set_fill_style_str(&particle.color.as_css());
            ctx.set_font(&format!("{}px 'JetBrains Mono', monospace", particle.size));
            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");
            let alpha = (particle.lifetime.min(1.0) * 255.0) as u8;
            if alpha > 30 {
                let _ = ctx.fill_text(
                    &particle.char.to_string(),
                    particle.x as f64,
                    particle.y as f64,
                );
            }
        }
    }

    // ASCII art for GAME OVER
    let game_over_art = r#"
 ██████╗  █████╗ ███╗   ███╗███████╗
██╔════╝ ██╔══██╗████╗ ████║██╔════╝
██║  ███╗███████║██╔████╔██║█████╗
██║   ██║██╔══██║██║╚██╔╝██║██╔══╝
╚██████╔╝██║  ██║██║ ╚═╝ ██║███████╗
 ╚═════╝ ╚═╝  ╚═╝╚═╝     ╚═╝╚══════╝
 ██████╗ ██╗   ██╗███████╗██████╗
██╔═══██╗██║   ██║██╔════╝██╔══██╗
██║   ██║██║   ██║█████╗  ██████╔╝
██║   ██║╚██╗ ██╔╝██╔══╝  ██╔══██╗
╚██████╔╝ ╚████╔╝ ███████╗██║  ██║
 ╚═════╝   ╚═══╝  ╚══════╝╚═╝  ╚═╝
"#;

    let lines: Vec<&str> = game_over_art.lines().filter(|l| !l.is_empty()).collect();
    let banner_height = lines.len() as f64 * 12.0;
    let start_y = h / 2.0 - banner_height / 2.0 - 60.0;

    ctx.set_font("10px 'JetBrains Mono', monospace");
    ctx.set_text_align("center");
    ctx.set_text_baseline("top");

    // Flicker effect
    let frame = state.frame();
    let flicker = if frame % 60 < 5 { 0.6 } else { 1.0 };

    for (i, line) in lines.iter().enumerate() {
        // Red gradient
        let intensity = 100 + (i as u32 * 10).min(155);
        ctx.set_fill_style_str(&format!("rgba({}, 50, 50, {})", intensity, flicker));
        let _ = ctx.fill_text(line, w / 2.0, start_y + i as f64 * 12.0);
    }

    // Message
    ctx.set_font(&format!("{}px 'JetBrains Mono', monospace", font_size));
    ctx.set_fill_style_str(&theme.info_text.as_css());
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text("Too many mistakes!", w / 2.0, h / 2.0 + 50.0);

    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', monospace",
        font_size * 0.7
    ));
    ctx.set_fill_style_str(&theme.info_text.as_css_alpha(0.8));
    let _ = ctx.fill_text(
        "Press N for new game, 1-6 for difficulty",
        w / 2.0,
        h / 2.0 + 90.0,
    );
}

/// Render new game menu
fn render_menu(
    ctx: &CanvasRenderingContext2d,
    theme: &Theme,
    width: u32,
    height: u32,
    font_size: f64,
) {
    // Semi-transparent overlay
    ctx.set_fill_style_str(&theme.background.as_css_alpha(0.9));
    ctx.fill_rect(0.0, 0.0, width as f64, height as f64);

    // Title
    ctx.set_font(&format!(
        "bold {}px 'JetBrains Mono', monospace",
        font_size * 1.5
    ));
    ctx.set_fill_style_str(&theme.given_text.as_css());
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text("NEW GAME", width as f64 / 2.0, height as f64 / 2.0 - 120.0);

    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', monospace",
        font_size * 0.9
    ));
    ctx.set_fill_style_str(&theme.info_text.as_css());

    let difficulties = [
        ("1", "Beginner"),
        ("2", "Easy"),
        ("3", "Medium"),
        ("4", "Intermediate"),
        ("5", "Hard"),
        ("6", "Expert"),
    ];

    let mut cy = height as f64 / 2.0 - 60.0;
    for (key, name) in difficulties {
        let _ = ctx.fill_text(&format!("[{}] {}", key, name), width as f64 / 2.0, cy);
        cy += font_size * 1.3;
    }

    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', monospace",
        font_size * 0.7
    ));
    ctx.set_fill_style_str(&theme.candidate_text.as_css());
    let _ = ctx.fill_text("Press Escape to cancel", width as f64 / 2.0, cy + 30.0);
}

/// Render temporary message
fn render_message(
    ctx: &CanvasRenderingContext2d,
    theme: &Theme,
    message: &str,
    width: u32,
    height: u32,
    font_size: f64,
) {
    let msg_y = height as f64 - 50.0;

    // Background
    ctx.set_fill_style_str(&theme.background.as_css_alpha(0.8));
    let metrics = ctx.measure_text(message).ok();
    let msg_width = metrics.map(|m| m.width()).unwrap_or(200.0) + 40.0;
    ctx.fill_rect(
        (width as f64 - msg_width) / 2.0,
        msg_y - font_size,
        msg_width,
        font_size * 2.0,
    );

    // Text
    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', monospace",
        font_size * 0.8
    ));
    ctx.set_fill_style_str(&theme.message_text.as_css());
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text(message, width as f64 / 2.0, msg_y);
}
