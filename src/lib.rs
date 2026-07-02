//! Macro OS Engines
//!
//! One Rust application with modularly separated engines:
//! - parse: ambiguity-tolerant inline macro parser
//! - context: context hierarchy/indexing
//! - navigation: typed alias/navigation resolver
//! - history: append-only event history and frequency scoring
//! - watchdog: directory/file-event rule planner

pub mod context;
pub mod database;
pub mod history;
pub mod navigation;
pub mod parse;
pub mod walk;
pub mod watchdog;
pub mod test_logging;