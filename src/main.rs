//! `claude-answers` — print your answers to Claude Code's questions.
//!
//! Usage:
//!   claude-answers                          newest transcript in this project
//!   claude-answers All                      every transcript in this project
//!   claude-answers '(Session 47318657)'     transcripts matching an id fragment
//!   claude-answers '(File /path/to.jsonl)'  one explicit transcript file
//!   claude-answers '(Grep (All Bluetooth))' filter answers by text
//!
//! The argument is a single NOTA record decoded into a `Query`; with no
//! argument the newest transcript is shown (as if `Latest` were given).

use std::io::Write;

use claude_answers::{ProjectDirectory, Query, Result};

fn main() -> Result<()> {
    let query = match std::env::args().nth(1) {
        Some(argument) => Query::parse(&argument)?,
        None => Query::Latest,
    };

    let project = ProjectDirectory::for_current_directory()?;
    let stdout = std::io::stdout();
    let mut writer = stdout.lock();
    let written = query.run(&project, &mut writer)?;
    if written == 0 {
        writeln!(writer, "(no answers found)")?;
    }
    Ok(())
}
