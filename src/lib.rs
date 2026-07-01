//! `claude-answers` — recall your answers to Claude Code's AskUserQuestion
//! prompts straight from the on-disk session transcripts.
//!
//! The compact `→ (notes only)` line Claude Code prints hides the option you
//! picked and the notes you typed. Both are kept in the session transcript
//! under `~/.claude/projects/<encoded-cwd>/<session>.jsonl`; this crate reads
//! them back.
//!
//! The single command-line argument is one NOTA record decoded into [`Query`];
//! see its documentation for the argument grammar.

pub mod error;
pub mod query;
pub mod transcript;

pub use error::{Error, Result};
pub use query::Query;
pub use transcript::{Answer, ProjectDirectory, Transcript};
