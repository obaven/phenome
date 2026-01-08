//! Shared formatting helpers used by UI and CLI.

mod assembly;
mod problems;

pub use assembly::{assembly_groups, AssemblyGroup, AssemblyStepInfo};
pub use problems::problem_lines;
