//! `claude-answers` — print your answers to Claude Code's questions.
//!
//! Usage:
//!   claude-answers                          newest transcript in this project
//!   claude-answers All                      every transcript in this project
//!   claude-answers Session.47318657          transcripts matching an id fragment
//!   claude-answers 'File.«/path/to.jsonl»'   one explicit transcript file
//!   claude-answers 'Grep.{ All Bluetooth }' filter answers by text
//!
//! The argument is one typed Datom `Query`; with no argument the newest
//! transcript is shown (as if `Latest` were given). Output is canonical Datom:
//! a vector of `{ question option notes }` answer structs. Faults print as
//! canonical Datom on stderr.

use std::process::ExitCode;

use datom_codec::Datomizable;
use protos::{Protosizable, Textualizable};

use claude_answers::{Answer, ProjectDirectory, Query};

fn main() -> ExitCode {
    let query = match std::env::args().nth(1) {
        Some(argument) => match claude_answers::parse(&argument) {
            Ok(query) => query,
            Err(claude_answers::Error::Argument(fault)) => {
                eprintln!("{}", fault.datomize(vec![]).protosize().textualize());
                return ExitCode::FAILURE;
            }
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::FAILURE;
            }
        },
        None => Query::Latest,
    };

    let project = match ProjectDirectory::for_current_directory() {
        Ok(project) => project,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };

    match query.run(&project) {
        Ok(answers) => {
            let datom: Vec<Answer> = answers;
            println!("{}", datom.datomize(vec![]).protosize().textualize());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
