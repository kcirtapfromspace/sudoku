//! Sudoku Core Engine
//!
//! A cross-platform Sudoku engine supporting puzzle generation, solving, hints,
//! and variant rules (X-Sudoku, Killer Sudoku, etc.)

mod bitset;
mod cell;
mod constraint;
mod diversity;
mod generator;
mod grid;
mod position;
mod solver;

pub use bitset::BitSet;
pub use cell::Cell;
pub use constraint::{
    BoxConstraint, ColumnConstraint, Constraint, ConstraintBox, DiagonalConstraint,
    KillerCageConstraint, RowConstraint, ThermoConstraint,
};
pub use diversity::{DifficultyStats, DiversityAnalyzer, DiversityReport, TheoreticalEstimates};
pub use generator::{Generator, GeneratorConfig, SymmetryType};
pub use grid::{Grid, MoveError, ValidationResult};
pub use position::Position;
pub use solver::{Difficulty, Hint, HintType, Solver, SolverConfig, Technique};
