//! Actualizing the Datom argument and running a query against a fixture project.

use std::path::{Path, PathBuf};

use datom_codec::Textualizable;
use protos::Text;

use claude_answers::{Answer, ProjectDirectory, Query};

fn text(value: &str) -> Text {
    value.try_into().expect("fixture text")
}

fn fixture_project() -> ProjectDirectory {
    let home = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/home");
    ProjectDirectory::locate(&home, Path::new("/w"))
}

fn run(query: &Query) -> Vec<Answer> {
    query.run(&fixture_project()).unwrap()
}

// ---------------------------------------------------------------------------
// Parse and round-trip tests
// ---------------------------------------------------------------------------

#[test]
fn latest_parses_from_a_bare_atom() {
    let query = claude_answers::parse("Latest").unwrap();
    assert!(matches!(query, Query::Latest));
    assert_eq!(query.textualize(), "Latest");
}

#[test]
fn all_parses_from_a_bare_atom() {
    let query = claude_answers::parse("All").unwrap();
    assert!(matches!(query, Query::All));
    assert_eq!(query.textualize(), "All");
}

#[test]
fn session_parses_with_an_id_fragment() {
    let query = claude_answers::parse("Session.47318657").unwrap();
    assert!(matches!(query, Query::Session(ref s) if s.as_ref() == "47318657"));
    assert_eq!(query.textualize(), "Session.47318657");
}

#[test]
fn file_parses_with_a_path() {
    let query = claude_answers::parse("File./home/li/x.jsonl").unwrap();
    assert!(matches!(query, Query::File(ref p) if p.as_ref() == "/home/li/x.jsonl"));
    assert_eq!(query.textualize(), "File./home/li/x.jsonl");
}

#[test]
fn grep_wraps_a_selection() {
    let query = claude_answers::parse("Grep.{ All Bluetooth }").unwrap();
    assert!(matches!(query, Query::Grep(_, ref needle) if needle.as_ref() == "Bluetooth"));
    assert_eq!(query.textualize(), "Grep.{ All Bluetooth }");
}

#[test]
fn grep_takes_curly_quoted_multiword_text() {
    let input = "Grep.{ Session.47318657 \u{201C}Bluetooth adapter\u{201D} }";
    let query = claude_answers::parse(input).unwrap();
    assert!(matches!(query, Query::Grep(_, ref needle) if needle.as_ref() == "Bluetooth adapter"));
    assert_eq!(query.textualize(), input);
}

#[test]
fn grep_round_trips_nested() {
    let input = "Grep.{ Grep.{ Session.47318657 Bluetooth } adapter }";
    let query = claude_answers::parse(input).unwrap();
    assert_eq!(query.textualize(), input);
}

// ---------------------------------------------------------------------------
// Execution tests
// ---------------------------------------------------------------------------

#[test]
fn all_returns_every_answer() {
    let answers = run(&Query::All);
    assert!(answers.len() >= 2);
    assert!(answers.iter().any(|a| a.0.contains("Bluetooth")));
    assert!(
        answers
            .iter()
            .any(|a| a.0.contains("What should the repo be named?"))
    );
}

#[test]
fn grep_keeps_only_matching_answers() {
    let query = claude_answers::parse("Grep.{ All Bluetooth }").unwrap();
    let answers = run(&query);
    assert!(answers.iter().all(|a| {
        a.0.to_lowercase().contains("bluetooth")
            || a.1.to_lowercase().contains("bluetooth")
            || a.2.to_lowercase().contains("bluetooth")
    }));
}

#[test]
fn latest_reads_the_single_fixture_transcript() {
    let answers = run(&Query::Latest);
    assert!(!answers.is_empty());
}

#[test]
fn explicit_file_reads_that_transcript() {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/home/.claude/projects/-w/session-11112222.jsonl");
    let query = claude_answers::parse(&format!("File.{}", path.display())).unwrap();
    let answers = run(&query);
    assert!(answers.iter().any(|a| a.0.contains("Bluetooth")));
}

// ---------------------------------------------------------------------------
// Datom output tests
// ---------------------------------------------------------------------------

#[test]
fn answer_textualizes_as_datom_struct() {
    let answer = Answer(text("Q?"), text("A"), text(""));
    let text = answer.textualize();
    // A struct of three texts: { Q? A "" }
    // "Q?" is not bare safe (contains ?) so it's curly-quoted
    assert!(text.contains("Q?"));
    assert!(text.contains("A"));
}

#[test]
fn answers_textualize_as_datom_vector() {
    let answers = vec![
        Answer(text("Q1"), text("A1"), text("")),
        Answer(text("Q2"), text("A2"), text("notes")),
    ];
    let text = answers.textualize();
    assert!(text.starts_with("[ "));
    assert!(text.ends_with(" ]"));
}

// ---------------------------------------------------------------------------
// Fault tests
// ---------------------------------------------------------------------------

#[test]
fn wrong_shapes_are_refused() {
    assert!(claude_answers::parse("Unknown").is_err());
    assert!(claude_answers::parse("Grep.All").is_err());
    assert!(claude_answers::parse("Grep.{ All }").is_err());
    assert!(claude_answers::parse("Grep.{ All Bluetooth extra }").is_err());
}
