//! Actualizing the Datom argument and running a query against a fixture project.

use std::path::{Path, PathBuf};

use datom_codec::Datomizable;
use protos::{Compactable, Protosizable};

use claude_answers::{Answer, ProjectDirectory, Query};

fn datom_text<T: Datomizable>(value: &T) -> String {
    value.datomize(vec![]).protosize().compact()
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
    assert_eq!(datom_text(&query), "Latest");
}

#[test]
fn all_parses_from_a_bare_atom() {
    let query = claude_answers::parse("All").unwrap();
    assert!(matches!(query, Query::All));
    assert_eq!(datom_text(&query), "All");
}

#[test]
fn session_parses_with_an_id_fragment() {
    let query = claude_answers::parse("Session.47318657").unwrap();
    assert!(matches!(query, Query::Session(ref s) if s.as_str() == "47318657"));
    assert_eq!(datom_text(&query), "Session.47318657");
}

#[test]
fn file_parses_with_a_path() {
    let query = claude_answers::parse("File./home/li/x.jsonl").unwrap();
    assert!(matches!(query, Query::File(ref p) if p.as_str() == "/home/li/x.jsonl"));
    assert_eq!(datom_text(&query), "File./home/li/x.jsonl");
}

#[test]
fn grep_wraps_a_selection() {
    let query = claude_answers::parse("Grep.{ All Bluetooth }").unwrap();
    assert!(matches!(query, Query::Grep(ref data) if data.string == "Bluetooth"));
    assert_eq!(datom_text(&query), "Grep.{ All Bluetooth }");
}

#[test]
fn grep_takes_guillemet_delimited_multiword_text() {
    let input = "Grep.{ Session.47318657 «Bluetooth adapter» }";
    let query = claude_answers::parse(input).unwrap();
    assert!(matches!(query, Query::Grep(ref data) if data.string == "Bluetooth adapter"));
    assert_eq!(datom_text(&query), input);
}

#[test]
fn grep_round_trips_nested() {
    let input = "Grep.{ Grep.{ Session.47318657 Bluetooth } adapter }";
    let query = claude_answers::parse(input).unwrap();
    assert_eq!(datom_text(&query), input);
}

// ---------------------------------------------------------------------------
// Execution tests
// ---------------------------------------------------------------------------

#[test]
fn all_returns_every_answer() {
    let answers = run(&Query::All);
    assert!(answers.len() >= 2);
    assert!(
        answers
            .iter()
            .any(|answer| answer.first_string.contains("Bluetooth"))
    );
    assert!(answers.iter().any(|answer| {
        answer
            .first_string
            .contains("What should the repo be named?")
    }));
}

#[test]
fn grep_keeps_only_matching_answers() {
    let query = claude_answers::parse("Grep.{ All Bluetooth }").unwrap();
    let answers = run(&query);
    assert!(answers.iter().all(|answer| {
        answer.first_string.to_lowercase().contains("bluetooth")
            || answer.second_string.to_lowercase().contains("bluetooth")
            || answer.third_string.to_lowercase().contains("bluetooth")
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
    let query = claude_answers::parse(&format!("File.«{}»", path.display())).unwrap();
    let answers = run(&query);
    assert!(
        answers
            .iter()
            .any(|answer| answer.first_string.contains("Bluetooth"))
    );
}

// ---------------------------------------------------------------------------
// Datom output tests
// ---------------------------------------------------------------------------

#[test]
fn answer_textualizes_as_datom_struct() {
    let answer = Answer {
        first_string: "Q?".to_owned(),
        second_string: "A".to_owned(),
        third_string: String::new(),
    };
    let text = datom_text(&answer);
    // A struct of three texts: { Q? A "" }
    // "Q?" is not bare safe (contains ?) so it is guillemet-delimited.
    assert!(text.contains("Q?"));
    assert!(text.contains("A"));
}

#[test]
fn answers_textualize_as_datom_vector() {
    let answers = vec![
        Answer {
            first_string: "Q1".to_owned(),
            second_string: "A1".to_owned(),
            third_string: String::new(),
        },
        Answer {
            first_string: "Q2".to_owned(),
            second_string: "A2".to_owned(),
            third_string: "notes".to_owned(),
        },
    ];
    let text = datom_text(&answers);
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

#[test]
fn query_archives_with_rkyv_and_reads_back_equal() {
    let query = Query::Grep(claude_answers::generated::Grep_Data {
        query: Box::new(Query::Session("47318657".to_string())),
        string: "two words".to_string(),
    });
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&query).expect("archive");
    let read = rkyv::from_bytes::<Query, rkyv::rancor::Error>(&bytes).expect("read back");
    assert_eq!(read, query);
}
