//! Locating a project directory and extracting answers from a transcript.

use std::path::{Path, PathBuf};

use claude_answers::{ProjectDirectory, Transcript};

fn fixture_transcript() -> Transcript {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/home/.claude/projects/-w/session-11112222.jsonl");
    Transcript::at(path)
}

#[test]
fn locate_replaces_every_nonalphanumeric_character_with_a_dash() {
    // Claude Code encodes the cwd by replacing every non-alphanumeric
    // character (not just `/`) with `-`, preserving case. A slash-only
    // encoding would leave the `.` in `github.com` and miss the directory.
    let project = ProjectDirectory::locate(
        Path::new("/home/li"),
        Path::new("/git/github.com/LiGoldragon/claude-answers"),
    );
    assert!(
        project
            .path()
            .ends_with(".claude/projects/-git-github-com-LiGoldragon-claude-answers")
    );
}

#[test]
fn answers_reads_option_and_multiline_notes() {
    let answers = fixture_transcript().answers().unwrap();
    assert_eq!(answers.len(), 2);

    let bluetooth = answers
        .iter()
        .find(|answer| answer.question.contains("Bluetooth"))
        .unwrap();
    assert_eq!(bluetooth.option, "(notes only)");
    assert!(bluetooth.notes.contains("active recording journal"));
    assert!(bluetooth.notes.contains("captured chunk immediately"));
}

#[test]
fn answers_reads_a_question_without_notes() {
    let answers = fixture_transcript().answers().unwrap();
    let named = answers
        .iter()
        .find(|answer| answer.option == "claude-answers")
        .unwrap();
    assert!(named.notes.is_empty());
}

#[test]
fn matches_is_case_insensitive_across_fields() {
    let answers = fixture_transcript().answers().unwrap();
    assert!(answers.iter().any(|answer| answer.matches("bluetooth")));
    assert!(
        answers
            .iter()
            .any(|answer| answer.matches("ACTIVE RECORDING"))
    );
    assert!(
        !answers
            .iter()
            .any(|answer| answer.matches("nonexistent-token"))
    );
}
