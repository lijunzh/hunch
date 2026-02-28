//! Match span and matching infrastructure.
//!
//! This module provides the core types ([`MatchSpan`], [`Property`]) and
//! the conflict resolution engine used by the pipeline. The [`rule_loader`]
//! and [`regex_utils`] sub-modules power the TOML-driven matching engine.
//!
//! > **Stability note:** [`Property`] and [`MatchSpan`] are stable public API.
//! > The `engine`, `rule_loader`, and `regex_utils` sub-modules are exported
//! > for advanced use but may change between minor versions.

pub mod engine;
pub mod regex_utils;
pub mod rule_loader;
pub mod span;

pub use engine::resolve_conflicts;
pub use span::{MatchSpan, Property};
