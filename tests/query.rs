//! Embodying the Datomic argument and running a query against a fixture project.

use std::path::{Path, PathBuf};

use claude_answers::{ProjectDirectory, Query};
use datomic::{Datomic, Text, TextEdge};

fn fixture_project() -> ProjectDirectory {
    // The fixture lives at tests/home/.claude/projects/-w/, i.e. home =
    // tests/home and working directory = /w (encoded "-w").
    let home = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/home");
    ProjectDirectory::locate(&home, Path::new("/w"))
}

fn render(query: &Query) -> String {
    let mut buffer = Vec::new();
    query.run(&fixture_project(), &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

#[test]
fn latest_parses_from_a_bare_atom() {
    let query = Text::<Query>::from("Latest")
        .embody()
        .expect("the public Datomic text edge embodies Latest");
    assert_eq!(query, Query::Latest);
    assert_eq!(query.textualize().as_ref(), "Latest");
}

#[test]
fn all_parses_from_a_bare_atom() {
    assert_eq!(Query::parse("All").unwrap(), Query::All);
}

#[test]
fn session_parses_with_an_id_fragment() {
    assert_eq!(
        Query::parse("Session.47318657").unwrap(),
        Query::session("47318657").expect("representable session fragment")
    );
}

#[test]
fn file_parses_with_a_path() {
    assert_eq!(
        Query::parse("File./home/li/x.jsonl").unwrap(),
        Query::file("/home/li/x.jsonl").expect("representable transcript path")
    );
}

#[test]
fn grep_wraps_a_selection() {
    assert_eq!(
        Query::parse("Grep.{All Bluetooth}").unwrap(),
        Query::grep(Query::All, "Bluetooth").expect("representable filter")
    );
}

#[test]
fn grep_takes_parenthesized_multiword_text() {
    assert_eq!(
        Query::parse("Grep.{Session.47318657 (Bluetooth adapter)}").unwrap(),
        Query::grep(
            Query::session("47318657").expect("representable session fragment"),
            "Bluetooth adapter",
        )
        .expect("representable filter")
    );
}

#[test]
fn all_prints_every_answer_with_a_count_footer() {
    let output = render(&Query::All);
    assert!(output.contains("Bluetooth mic drops"));
    assert!(output.contains("What should the repo be named?"));
    assert!(output.contains("# ^ 2 answer(s) in session-11112222.jsonl"));
}

#[test]
fn grep_keeps_only_matching_answers() {
    let output = render(&Query::parse("Grep.{All Bluetooth}").unwrap());
    assert!(output.contains("Bluetooth mic drops"));
    assert!(!output.contains("What should the repo be named?"));
    assert!(output.contains("# ^ 1 answer(s) in"));
}

#[test]
fn latest_reads_the_single_fixture_transcript() {
    let output = render(&Query::Latest);
    assert!(output.contains("Bluetooth mic drops"));
}

#[test]
fn explicit_file_reads_that_transcript() {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/home/.claude/projects/-w/session-11112222.jsonl");
    let query =
        Query::file(path.to_string_lossy().into_owned()).expect("fixture path is representable");
    let mut buffer = Vec::new();
    query.run(&fixture_project(), &mut buffer).unwrap();
    assert!(
        String::from_utf8(buffer)
            .unwrap()
            .contains("Bluetooth mic drops")
    );
}
