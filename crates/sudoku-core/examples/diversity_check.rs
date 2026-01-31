//! Puzzle Diversity Analysis Tool
//!
//! Analyzes how many unique Sudoku puzzles can be generated at each difficulty level.
//!
//! Run with: cargo run --example diversity_check -- [samples_per_difficulty]

use std::time::Instant;
use sudoku_core::{Difficulty, DiversityAnalyzer, TheoreticalEstimates};

fn main() {
    let samples = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);

    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║           Sudoku Puzzle Diversity Analysis                           ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");
    println!();

    // Print theoretical estimates
    let theoretical = TheoreticalEstimates::new();
    println!("📊 Theoretical Sudoku Mathematics:");
    println!("   ├─ Total valid completed grids: ~6.67 × 10²¹");
    println!("   ├─ Essentially unique grids:    ~5.47 × 10⁹");
    println!("   ├─ Minimum clues for unique:    17");
    println!("   └─ Known 17-clue puzzles:       ~49,000");
    println!();

    println!("📈 Estimated Puzzles by Difficulty (theoretical):");
    println!("   ┌─────────────────┬─────────────┬─────────────────────────┐");
    println!("   │ Difficulty      │ Clue Range  │ Estimated Puzzles       │");
    println!("   ├─────────────────┼─────────────┼─────────────────────────┤");

    let difficulties = [
        (Difficulty::Beginner, "Beginner", "45-55"),
        (Difficulty::Easy, "Easy", "36-45"),
        (Difficulty::Medium, "Medium", "32-38"),
        (Difficulty::Intermediate, "Intermediate", "28-34"),
        (Difficulty::Hard, "Hard", "24-30"),
        (Difficulty::Expert, "Expert", "22-26"),
        (Difficulty::Master, "Master", "20-24"),
        (Difficulty::Extreme, "Extreme", "17-22"),
    ];

    for (diff, name, range) in &difficulties {
        let estimate = theoretical.estimate_for_difficulty(*diff);
        println!(
            "   │ {:15} │ {:11} │ {:>23} │",
            name,
            range,
            format_number(estimate)
        );
    }
    println!("   └─────────────────┴─────────────┴─────────────────────────┘");
    println!();

    // Run empirical sampling
    println!(
        "🔬 Empirical Sampling ({} puzzles per difficulty)...",
        samples
    );
    println!();

    let mut analyzer = DiversityAnalyzer::new();
    let start = Instant::now();

    // Only test standard difficulties (not Master/Extreme which are slow)
    let test_difficulties = [
        (Difficulty::Beginner, "Beginner"),
        (Difficulty::Easy, "Easy"),
        (Difficulty::Medium, "Medium"),
        (Difficulty::Intermediate, "Intermediate"),
        (Difficulty::Hard, "Hard"),
        (Difficulty::Expert, "Expert"),
    ];

    println!("   ┌─────────────────┬─────────┬──────────┬───────────┬─────────────────┐");
    println!("   │ Difficulty      │ Samples │ Unique   │ Avg Clues │ Uniqueness Rate │");
    println!("   ├─────────────────┼─────────┼──────────┼───────────┼─────────────────┤");

    for (diff, name) in &test_difficulties {
        let diff_start = Instant::now();
        let stats = analyzer.analyze_difficulty(*diff, samples);
        let elapsed = diff_start.elapsed();

        let uniqueness_rate = if stats.samples > 0 {
            100.0 * stats.unique_puzzles as f64 / stats.samples as f64
        } else {
            0.0
        };

        println!(
            "   │ {:15} │ {:7} │ {:8} │ {:9.1} │ {:14.1}% │  ({:.1}s)",
            name,
            stats.samples,
            stats.unique_puzzles,
            stats.avg_clues,
            uniqueness_rate,
            elapsed.as_secs_f64()
        );
    }

    println!("   └─────────────────┴─────────┴──────────┴───────────┴─────────────────┘");
    println!();

    let total_elapsed = start.elapsed();
    println!(
        "⏱  Total analysis time: {:.2}s",
        total_elapsed.as_secs_f64()
    );
    println!();

    // Print the full summary
    println!("{}", analyzer.summary());

    println!();
    println!("💡 Interpretation:");
    println!("   • 100% uniqueness rate → extremely large puzzle space");
    println!("   • The generator produces effectively unlimited unique puzzles");
    println!("   • Harder difficulties have fewer possible puzzles due to:");
    println!("     - Fewer clue combinations");
    println!("     - Stricter uniqueness requirements");
    println!("     - Specific technique requirements");
}

fn format_number(n: u128) -> String {
    if n == 0 {
        return "0".to_string();
    }
    if n < 1_000_000 {
        return format!("{:>15}", n);
    }

    let log10 = (n as f64).log10().floor() as u32;
    let mantissa = n as f64 / 10_f64.powi(log10 as i32);

    format!("{:.2} × 10^{}", mantissa, log10)
}
