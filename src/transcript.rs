//! Reading Claude Code session transcripts and the answers inside them.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_json::Value;

use crate::error::{Error, Result};

/// The `~/.claude/projects/<encoded-cwd>` directory holding a project's
/// session transcripts. Claude Code encodes the working directory by
/// replacing every path separator with `-`.
pub struct ProjectDirectory {
    path: PathBuf,
}

impl ProjectDirectory {
    /// The transcript directory for the current working directory.
    pub fn for_current_directory() -> Result<Self> {
        let home = std::env::var_os("HOME").ok_or(Error::HomeNotSet)?;
        let working_directory = std::env::current_dir()?;
        Ok(Self::locate(Path::new(&home), &working_directory))
    }

    /// Pure resolver: the transcript directory for `home` and `working_directory`.
    pub fn locate(home: &Path, working_directory: &Path) -> Self {
        let encoded = working_directory.to_string_lossy().replace('/', "-");
        let path = home.join(".claude").join("projects").join(encoded);
        Self { path }
    }

    /// The directory path itself.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Every `*.jsonl` transcript, oldest first (by modification time).
    pub fn transcripts_by_age(&self) -> Result<Vec<PathBuf>> {
        let read = match fs::read_dir(&self.path) {
            Ok(read) => read,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(Error::NoTranscripts(self.path.clone()));
            }
            Err(error) => return Err(error.into()),
        };

        let mut dated: Vec<(std::time::SystemTime, PathBuf)> = Vec::new();
        for entry in read {
            let entry = entry?;
            let path = entry.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "jsonl")
            {
                dated.push((entry.metadata()?.modified()?, path));
            }
        }

        if dated.is_empty() {
            return Err(Error::NoTranscripts(self.path.clone()));
        }
        dated.sort_by_key(|(modified, _)| *modified);
        Ok(dated.into_iter().map(|(_, path)| path).collect())
    }

    /// Transcripts whose file name contains `fragment`, oldest first.
    pub fn transcripts_matching(&self, fragment: &str) -> Result<Vec<PathBuf>> {
        let matches: Vec<PathBuf> = self
            .transcripts_by_age()?
            .into_iter()
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.contains(fragment))
            })
            .collect();
        if matches.is_empty() {
            return Err(Error::SessionNotFound {
                fragment: fragment.to_owned(),
                directory: self.path.clone(),
            });
        }
        Ok(matches)
    }
}

/// One session transcript file.
pub struct Transcript {
    path: PathBuf,
}

impl Transcript {
    /// A transcript at the given path.
    pub fn at(path: PathBuf) -> Self {
        Self { path }
    }

    /// The file's display name, used in output headers.
    pub fn name(&self) -> Cow<'_, str> {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_else(|| self.path.to_string_lossy())
    }

    /// Every AskUserQuestion answer recorded in this transcript, in file order.
    /// Lines that are not answer records are skipped, so unrelated tool output
    /// never trips the reader.
    pub fn answers(&self) -> Result<Vec<Answer>> {
        let text = fs::read_to_string(&self.path)?;
        let mut answers = Vec::new();
        for line in text.lines() {
            if !line.contains("\"answers\"") {
                continue;
            }
            let Ok(record) = serde_json::from_str::<TranscriptRecord>(line) else {
                continue;
            };
            let Some(result) = record.tool_use_result else {
                continue;
            };
            for (question, option) in &result.answers {
                let notes = result
                    .annotations
                    .get(question)
                    .map(|annotation| annotation.notes.clone())
                    .unwrap_or_default();
                answers.push(Answer::new(question.clone(), option, notes));
            }
        }
        Ok(answers)
    }
}

/// One answered question: the prompt, the option picked, and any typed notes.
pub struct Answer {
    pub question: String,
    pub option: String,
    pub notes: String,
}

impl Answer {
    /// Build an answer, rendering the raw option value to display text: a plain
    /// string as-is, a multi-select array joined with commas.
    pub fn new(question: String, option: &Value, notes: String) -> Self {
        let option = match option {
            Value::String(text) => text.clone(),
            Value::Array(items) => items
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", "),
            other => other.to_string(),
        };
        Self {
            question,
            option,
            notes,
        }
    }

    /// Whether `needle` appears (case-insensitively) in the question, the
    /// chosen option, or the notes.
    pub fn matches(&self, needle: &str) -> bool {
        let needle = needle.to_lowercase();
        self.question.to_lowercase().contains(&needle)
            || self.option.to_lowercase().contains(&needle)
            || self.notes.to_lowercase().contains(&needle)
    }

    /// Write the answer in the human-readable block form.
    pub fn render(&self, writer: &mut impl Write) -> io::Result<()> {
        writeln!(writer, "Q: {}", self.question)?;
        writeln!(writer, "  answer: {}", self.option)?;
        if !self.notes.is_empty() {
            writeln!(writer, "  notes: |")?;
            for line in self.notes.lines() {
                writeln!(writer, "    {line}")?;
            }
        }
        writeln!(writer)
    }
}

#[derive(Deserialize)]
struct TranscriptRecord {
    #[serde(rename = "toolUseResult")]
    tool_use_result: Option<ToolUseResult>,
}

#[derive(Deserialize)]
struct ToolUseResult {
    #[serde(default)]
    answers: BTreeMap<String, Value>,
    #[serde(default)]
    annotations: BTreeMap<String, Annotation>,
}

#[derive(Deserialize)]
struct Annotation {
    #[serde(default)]
    notes: String,
}
