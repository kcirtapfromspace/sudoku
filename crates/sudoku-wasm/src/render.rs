//! Canvas rendering for terminal-like Sudoku UI

use crate::game::{GameState, HintDetailLevel, InputMode, ScreenState, MAX_MISTAKES};
use crate::theme::{Color, Theme};
use sudoku_core::{Hint, Polarity, Position, ProofCertificate};
use web_sys::CanvasRenderingContext2d;

/// Role of a cell in the current hint visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HintCellRole {
    None,
    /// Target cell (placement or elimination)
    Target,
    /// Generic involved cell
    Involved,
    /// AIC ON-polarity node
    ChainOn,
    /// AIC OFF-polarity node
    ChainOff,
    /// Fish base sector cell
    FishBase,
    /// Fish cover sector cell
    FishCover,
    /// Fish fin cell
    FishFin,
    /// UR floor (bivalue cell)
    UrFloor,
    /// UR roof (extra candidates)
    UrRoof,
    /// ALS group member
    AlsGroup,
}

/// Return cells belonging to a sector index.
/// Convention: 0..8=rows, 9..17=cols, 18..26=boxes.
fn sector_cells(sector: usize) -> Vec<usize> {
    if sector < 9 {
        // Row
        let row = sector;
        (0..9).map(|col| row * 9 + col).collect()
    } else if sector < 18 {
        // Column
        let col = sector - 9;
        (0..9).map(|row| row * 9 + col).collect()
    } else {
        // Box
        let b = sector - 18;
        let br = (b / 3) * 3;
        let bc = (b % 3) * 3;
        let mut cells = Vec::with_capacity(9);
        for r in br..br + 3 {
            for c in bc..bc + 3 {
                cells.push(r * 9 + c);
            }
        }
        cells
    }
}

/// Compute hint cell roles for every cell based on current hint and detail level.
fn compute_hint_roles(hint: &Hint, detail: HintDetailLevel) -> [HintCellRole; 81] {
    let mut roles = [HintCellRole::None; 81];

    // Always mark the target cell
    let target_idx = match &hint.hint_type {
        sudoku_core::HintType::SetValue { pos, .. }
        | sudoku_core::HintType::EliminateCandidates { pos, .. } => pos.row * 9 + pos.col,
    };
    roles[target_idx] = HintCellRole::Target;

    // Mark involved cells
    for pos in &hint.involved_cells {
        let idx = pos.row * 9 + pos.col;
        if roles[idx] == HintCellRole::None {
            roles[idx] = HintCellRole::Involved;
        }
    }

    // At ProofDetail, override with proof-specific roles
    if detail == HintDetailLevel::ProofDetail {
        if let Some(ref proof) = hint.proof {
            match proof {
                ProofCertificate::Fish {
                    base_sectors,
                    cover_sectors,
                    fins,
                    ..
                } => {
                    for &s in base_sectors {
                        for idx in sector_cells(s) {
                            if roles[idx] == HintCellRole::Involved {
                                roles[idx] = HintCellRole::FishBase;
                            }
                        }
                    }
                    for &s in cover_sectors {
                        for idx in sector_cells(s) {
                            if roles[idx] == HintCellRole::Involved {
                                roles[idx] = HintCellRole::FishCover;
                            }
                        }
                    }
                    for &idx in fins {
                        if idx < 81 {
                            roles[idx] = HintCellRole::FishFin;
                        }
                    }
                }
                ProofCertificate::Aic { chain, .. } => {
                    for &(cell, _digit, polarity) in chain {
                        if cell < 81 {
                            roles[cell] = match polarity {
                                Polarity::On => HintCellRole::ChainOn,
                                Polarity::Off => HintCellRole::ChainOff,
                            };
                        }
                    }
                }
                ProofCertificate::Uniqueness {
                    floor_cells,
                    roof_cells,
                    ..
                } => {
                    for &idx in floor_cells {
                        if idx < 81 {
                            roles[idx] = HintCellRole::UrFloor;
                        }
                    }
                    for &idx in roof_cells {
                        if idx < 81 {
                            roles[idx] = HintCellRole::UrRoof;
                        }
                    }
                }
                ProofCertificate::Als { als_chain, .. } => {
                    for als in als_chain {
                        for &idx in &als.cells {
                            if idx < 81 && roles[idx] != HintCellRole::Target {
                                roles[idx] = HintCellRole::AlsGroup;
                            }
                        }
                    }
                }
                ProofCertificate::Basic { .. }
                | ProofCertificate::Forcing { .. }
                | ProofCertificate::Backtracking => {}
            }
        }
    }

    // Ensure target stays as Target
    roles[target_idx] = HintCellRole::Target;
    roles
}

/// Map a hint cell role to the appropriate theme color.
fn role_color<'a>(role: HintCellRole, theme: &'a Theme) -> Option<&'a Color> {
    match role {
        HintCellRole::None => None,
        HintCellRole::Target => Some(&theme.hint_target_bg),
        HintCellRole::Involved => Some(&theme.hint_involved_bg),
        HintCellRole::ChainOn => Some(&theme.hint_chain_on),
        HintCellRole::ChainOff => Some(&theme.hint_chain_off),
        HintCellRole::FishBase => Some(&theme.hint_fish_base),
        HintCellRole::FishCover => Some(&theme.hint_fish_cover),
        HintCellRole::FishFin => Some(&theme.hint_fish_fin),
        HintCellRole::UrFloor => Some(&theme.hint_ur_floor),
        HintCellRole::UrRoof => Some(&theme.hint_ur_roof),
        HintCellRole::AlsGroup => Some(&theme.hint_als_group),
    }
}

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

            // Hint panel below grid when hint is active
            if state.current_hint().is_some() {
                let panel_y = grid_y + grid_height + 40.0;
                render_hint_panel(ctx, state, theme, grid_x, panel_y, grid_width, font_size);
            }

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
            render_menu(ctx, state, theme, width, height, font_size);
        }
        ScreenState::Stats => {
            render_grid(ctx, state, theme, grid_x, grid_y, cell_size, font_size);
            render_stats_screen(ctx, state, theme, width, height, font_size);
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

    // Pre-compute hint cell roles once per render
    let hint_roles: Option<[HintCellRole; 81]> = state
        .current_hint()
        .map(|hint| compute_hint_roles(hint, state.hint_detail()));

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
            // Priority: cursor > hint_target > proof_role > hint_involved > same_value > highlight > cell_bg
            let idx = row * 9 + col;
            let hint_role = hint_roles.map(|r| r[idx]).unwrap_or(HintCellRole::None);

            let bg_color = if pos == cursor {
                &theme.cursor_bg
            } else if let Some(color) = role_color(hint_role, theme) {
                color
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

    let _ = ctx.fill_text(
        &format!("{} (SE {:.1})", state.difficulty(), state.se_rating()),
        x,
        cy,
    );
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
        "p  n  S     Pause/New/Stats",
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
        "P/Space: Resume    S: Statistics",
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

    // Dim the grid behind the overlay
    ctx.set_fill_style_str(&theme.background.as_css_alpha(0.75));
    ctx.fill_rect(0.0, 0.0, w, h);

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
                ctx.set_fill_style_str(&color.as_css_alpha(0.15));
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

        // Content backdrop panel
        let panel_w = w * 0.75;
        let panel_x = (w - panel_w) / 2.0;
        let panel_top = h / 2.0 - 30.0;
        let panel_bottom = h / 2.0 + 175.0;
        ctx.set_fill_style_str(&theme.background.as_css_alpha(0.85));
        ctx.fill_rect(panel_x, panel_top, panel_w, panel_bottom - panel_top);

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
        // Fallback: simple overlay (already dimmed above)
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

    // Puzzle Universe progress
    let stats = state.player_stats();
    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', monospace",
        font_size * 0.65
    ));
    ctx.set_fill_style_str(&theme.message_text.as_css_alpha(0.9));
    let _ = ctx.fill_text(
        &format!("✨ Universe explored: {}", stats.universe_explored_text()),
        w / 2.0,
        h / 2.0 + 100.0,
    );

    ctx.set_font(&format!(
        "italic {}px 'JetBrains Mono', monospace",
        font_size * 0.55
    ));
    ctx.set_fill_style_str(&theme.candidate_text.as_css_alpha(0.7));
    let _ = ctx.fill_text(stats.universe_progress_note(), w / 2.0, h / 2.0 + 125.0);

    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', monospace",
        font_size * 0.7
    ));
    ctx.set_fill_style_str(&theme.info_text.as_css_alpha(0.8));
    let diff_hint = if state.secrets_unlocked() {
        "N: New game  1-8: Difficulty  S: Full stats"
    } else {
        "N: New game  1-6: Difficulty  S: Full stats"
    };
    let _ = ctx.fill_text(diff_hint, w / 2.0, h / 2.0 + 160.0);
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

    // Dark overlay to dim the grid
    ctx.set_fill_style_str(&theme.background.as_css_alpha(0.75));
    ctx.fill_rect(0.0, 0.0, w, h);
    ctx.set_fill_style_str(&theme.lose_color.as_css_alpha(0.15));
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

    // Content backdrop panel
    let panel_w = w * 0.75;
    let panel_x = (w - panel_w) / 2.0;
    let panel_top = h / 2.0 + 25.0;
    let panel_bottom = h / 2.0 + 110.0;
    ctx.set_fill_style_str(&theme.background.as_css_alpha(0.85));
    ctx.fill_rect(panel_x, panel_top, panel_w, panel_bottom - panel_top);

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
    let diff_hint = if state.secrets_unlocked() {
        "Press N for new game, 1-8 for difficulty"
    } else {
        "Press N for new game, 1-6 for difficulty"
    };
    let _ = ctx.fill_text(diff_hint, w / 2.0, h / 2.0 + 90.0);
}

/// Render new game menu
fn render_menu(
    ctx: &CanvasRenderingContext2d,
    state: &GameState,
    theme: &Theme,
    width: u32,
    height: u32,
    font_size: f64,
) {
    // Semi-transparent overlay
    ctx.set_fill_style_str(&theme.background.as_css_alpha(0.9));
    ctx.fill_rect(0.0, 0.0, width as f64, height as f64);

    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");

    let mut difficulties: Vec<(&str, &str)> = vec![
        ("1", "Beginner"),
        ("2", "Easy"),
        ("3", "Medium"),
        ("4", "Intermediate"),
        ("5", "Hard"),
        ("6", "Expert"),
    ];

    if state.secrets_unlocked() {
        difficulties.push(("7", "Master"));
        difficulties.push(("8", "Extreme"));
    }

    let line_h = font_size * 1.3;
    let list_height = difficulties.len() as f64 * line_h;
    let footer_gap = font_size * 1.5;
    // Total block: title + gap + list + gap + footer, centered vertically
    let title_space = font_size * 2.5;
    let total = title_space + list_height + footer_gap;
    let top = (height as f64 - total) / 2.0;

    ctx.set_font(&format!(
        "bold {}px 'JetBrains Mono', monospace",
        font_size * 1.5
    ));
    ctx.set_fill_style_str(&theme.given_text.as_css());
    let _ = ctx.fill_text("NEW GAME", width as f64 / 2.0, top);

    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', monospace",
        font_size * 0.9
    ));
    ctx.set_fill_style_str(&theme.info_text.as_css());

    let mut cy = top + title_space;
    for (key, name) in &difficulties {
        let _ = ctx.fill_text(&format!("[{}] {}", key, name), width as f64 / 2.0, cy);
        cy += line_h;
    }

    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', monospace",
        font_size * 0.7
    ));
    ctx.set_fill_style_str(&theme.candidate_text.as_css());
    let _ = ctx.fill_text(
        "[S] Statistics    [Esc] Cancel",
        width as f64 / 2.0,
        cy + footer_gap * 0.5,
    );
}

/// Render statistics screen with Puzzle Universe
fn render_stats_screen(
    ctx: &CanvasRenderingContext2d,
    state: &GameState,
    theme: &Theme,
    width: u32,
    height: u32,
    font_size: f64,
) {
    let w = width as f64;
    let h = height as f64;
    let stats = state.player_stats();

    // Semi-transparent overlay
    ctx.set_fill_style_str(&theme.background.as_css_alpha(0.95));
    ctx.fill_rect(0.0, 0.0, w, h);

    let line_height = font_size * 1.3;
    let small_font = font_size * 0.7;

    // Title
    ctx.set_font(&format!(
        "bold {}px 'JetBrains Mono', monospace",
        font_size * 1.5
    ));
    ctx.set_fill_style_str(&theme.given_text.as_css());
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    let _ = ctx.fill_text("STATISTICS", w / 2.0, 60.0);

    let mut cy = 110.0;
    let left_x = w / 2.0 - 180.0;
    let right_x = w / 2.0 + 180.0;

    ctx.set_text_align("left");

    // Overview section
    ctx.set_font(&format!(
        "bold {}px 'JetBrains Mono', monospace",
        font_size * 0.9
    ));
    ctx.set_fill_style_str(&theme.win_color.as_css());
    let _ = ctx.fill_text("Overview", left_x, cy);
    cy += line_height;

    ctx.set_font(&format!("{}px 'JetBrains Mono', monospace", small_font));
    ctx.set_fill_style_str(&theme.info_text.as_css());

    let _ = ctx.fill_text("Games Played:", left_x, cy);
    ctx.set_text_align("right");
    let _ = ctx.fill_text(&format!("{}", stats.games_played), right_x, cy);
    ctx.set_text_align("left");
    cy += line_height * 0.8;

    let _ = ctx.fill_text("Games Won:", left_x, cy);
    ctx.set_text_align("right");
    let _ = ctx.fill_text(&format!("{}", stats.games_won), right_x, cy);
    ctx.set_text_align("left");
    cy += line_height * 0.8;

    let _ = ctx.fill_text("Win Rate:", left_x, cy);
    ctx.set_text_align("right");
    let _ = ctx.fill_text(&format!("{:.1}%", stats.win_rate()), right_x, cy);
    ctx.set_text_align("left");
    cy += line_height * 0.8;

    let _ = ctx.fill_text("Total Play Time:", left_x, cy);
    ctx.set_text_align("right");
    let _ = ctx.fill_text(&stats.total_time_formatted(), right_x, cy);
    ctx.set_text_align("left");
    cy += line_height;

    // Streaks section
    ctx.set_font(&format!(
        "bold {}px 'JetBrains Mono', monospace",
        font_size * 0.9
    ));
    ctx.set_fill_style_str(&theme.win_color.as_css());
    let _ = ctx.fill_text("Streaks", left_x, cy);
    cy += line_height;

    ctx.set_font(&format!("{}px 'JetBrains Mono', monospace", small_font));
    ctx.set_fill_style_str(&theme.info_text.as_css());

    let _ = ctx.fill_text("Current Streak:", left_x, cy);
    ctx.set_text_align("right");
    let _ = ctx.fill_text(&format!("{}", stats.current_streak), right_x, cy);
    ctx.set_text_align("left");
    cy += line_height * 0.8;

    let _ = ctx.fill_text("Best Streak:", left_x, cy);
    ctx.set_text_align("right");
    let _ = ctx.fill_text(&format!("{}", stats.best_streak), right_x, cy);
    ctx.set_text_align("left");
    cy += line_height * 1.2;

    // Puzzle Universe section
    ctx.set_font(&format!(
        "bold {}px 'JetBrains Mono', monospace",
        font_size * 0.9
    ));
    ctx.set_fill_style_str(&theme.message_text.as_css());
    let _ = ctx.fill_text("✨ Puzzle Universe", left_x, cy);
    cy += line_height;

    ctx.set_font(&format!("{}px 'JetBrains Mono', monospace", small_font));
    ctx.set_fill_style_str(&theme.info_text.as_css());

    let _ = ctx.fill_text("Universe Explored:", left_x, cy);
    cy += line_height * 0.8;

    // Universe explored text (indented, monospace)
    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', monospace",
        small_font * 0.9
    ));
    ctx.set_fill_style_str(&theme.candidate_text.as_css());
    let _ = ctx.fill_text(&format!("  {}", stats.universe_explored_text()), left_x, cy);
    cy += line_height * 0.8;

    // Progress bar
    let bar_x = left_x + 20.0;
    let bar_width = 300.0;
    let bar_height = 8.0;

    ctx.set_fill_style_str(&theme.cell_bg.as_css());
    ctx.fill_rect(bar_x, cy, bar_width, bar_height);

    // The actual progress would be invisibly small, so show a tiny sliver
    if stats.games_won > 0 {
        ctx.set_fill_style_str(&theme.win_color.as_css());
        let progress_width = (bar_width * 0.002).max(2.0); // At least 2px
        ctx.fill_rect(bar_x, cy, progress_width, bar_height);
    }
    cy += line_height;

    // Cheeky progress note
    ctx.set_font(&format!(
        "italic {}px 'JetBrains Mono', monospace",
        small_font * 0.85
    ));
    ctx.set_fill_style_str(&theme.candidate_text.as_css_alpha(0.8));
    let _ = ctx.fill_text(stats.universe_progress_note(), left_x, cy);
    cy += line_height * 1.2;

    // Time to complete all
    ctx.set_font(&format!("{}px 'JetBrains Mono', monospace", small_font));
    ctx.set_fill_style_str(&theme.info_text.as_css());
    let _ = ctx.fill_text("Time to Complete All:", left_x, cy);
    cy += line_height * 0.8;

    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', monospace",
        small_font * 0.9
    ));
    ctx.set_fill_style_str(&theme.candidate_text.as_css());
    let _ = ctx.fill_text(&format!("  {}", stats.time_to_complete_text()), left_x, cy);
    cy += line_height * 0.8;

    // Cheeky time note
    ctx.set_font(&format!(
        "italic {}px 'JetBrains Mono', monospace",
        small_font * 0.85
    ));
    ctx.set_fill_style_str(&theme.candidate_text.as_css_alpha(0.8));
    let _ = ctx.fill_text(stats.time_note(), left_x, cy);

    // Instructions at bottom
    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', monospace",
        font_size * 0.7
    ));
    ctx.set_fill_style_str(&theme.candidate_text.as_css());
    ctx.set_text_align("center");
    let _ = ctx.fill_text("Press Escape or S to return", w / 2.0, h - 40.0);
}

/// Render hint info panel below the grid
fn render_hint_panel(
    ctx: &CanvasRenderingContext2d,
    state: &GameState,
    theme: &Theme,
    x: f64,
    y: f64,
    width: f64,
    font_size: f64,
) {
    let hint = match state.current_hint() {
        Some(h) => h,
        None => return,
    };

    let panel_height = font_size * 3.0;
    let padding = 8.0;
    let line_height = font_size * 0.75;
    let small_font = font_size * 0.55;

    // Panel background
    ctx.set_fill_style_str(&theme.hint_panel_bg.as_css());
    ctx.fill_rect(x, y, width, panel_height);

    // Technique name + SE rating
    ctx.set_text_align("left");
    ctx.set_text_baseline("top");
    ctx.set_font(&format!(
        "bold {}px 'JetBrains Mono', monospace",
        small_font
    ));
    ctx.set_fill_style_str(&theme.hint_technique_text.as_css());
    let header = format!("{} (SE {:.1})", hint.technique, hint.technique.se_rating());
    let _ = ctx.fill_text(&header, x + padding, y + padding);

    // Explanation text
    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', monospace",
        small_font * 0.9
    ));
    ctx.set_fill_style_str(&theme.hint_explain_text.as_css());
    let _ = ctx.fill_text(&hint.explanation, x + padding, y + padding + line_height);

    // Proof summary line (only at ProofDetail)
    if state.hint_detail() == HintDetailLevel::ProofDetail {
        if let Some(ref proof) = hint.proof {
            let proof_summary = match proof {
                ProofCertificate::Basic { kind } => format!("Proof: {}", kind),
                ProofCertificate::Fish {
                    digit,
                    base_sectors,
                    fins,
                    ..
                } => {
                    if fins.is_empty() {
                        format!(
                            "Fish on digit {}, {} base sectors",
                            digit,
                            base_sectors.len()
                        )
                    } else {
                        format!("Finned fish on digit {}, {} fins", digit, fins.len())
                    }
                }
                ProofCertificate::Aic {
                    chain, link_types, ..
                } => {
                    format!("Chain: {} nodes, {} links", chain.len(), link_types.len())
                }
                ProofCertificate::Als {
                    als_chain,
                    rcc_values,
                    ..
                } => {
                    format!(
                        "ALS chain: {} sets, {} RCC values",
                        als_chain.len(),
                        rcc_values.len()
                    )
                }
                ProofCertificate::Uniqueness { pattern, .. } => {
                    format!("Uniqueness: {}", pattern)
                }
                ProofCertificate::Forcing { branches, .. } => {
                    format!("Forcing: {} branches converge", branches)
                }
                ProofCertificate::Backtracking => "Backtracking".to_string(),
            };
            ctx.set_fill_style_str(&theme.hint_explain_text.as_css_alpha(0.7));
            let _ = ctx.fill_text(&proof_summary, x + padding, y + padding + line_height * 2.0);
        }
    }

    // Right-aligned prompt
    ctx.set_text_align("right");
    ctx.set_font(&format!(
        "{}px 'JetBrains Mono', monospace",
        small_font * 0.85
    ));
    ctx.set_fill_style_str(&theme.hint_technique_text.as_css_alpha(0.6));
    let prompt = if state.hint_detail() == HintDetailLevel::ProofDetail {
        "[proof shown]"
    } else {
        "[? for details]"
    };
    let _ = ctx.fill_text(prompt, x + width - padding, y + padding);
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
