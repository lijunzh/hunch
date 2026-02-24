//! Match span and matching infrastructure.

pub mod engine;
pub mod regex_utils;
pub mod span;

pub use engine::resolve_conflicts;
pub use regex_utils::captures_iter;
pub use span::{MatchSpan, Property};
