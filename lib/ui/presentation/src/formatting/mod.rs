//! Shared formatting helpers used by UI and CLI.

mod assembly;
mod problems;

pub use assembly::{AssemblyGroup, AssemblyStepInfo, assembly_groups};
pub use problems::problem_lines;
