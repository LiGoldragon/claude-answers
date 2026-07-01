//! The command-line argument surface: one NOTA record decoded into a [`Query`].

use std::io::Write;
use std::path::PathBuf;

use nota::{NotaDecode, NotaEncode, NotaSource};

use crate::error::Result;
use crate::transcript::{ProjectDirectory, Transcript};

/// The `claude-answers` argument, given as a single NOTA record.
///
/// ```text
/// Latest                                   newest transcript in this project
/// All                                      every transcript in this project
/// (Session 47318657)                       transcripts whose file name holds the id
/// (File /path/to.jsonl)                    one explicit transcript file
/// (Grep (All Bluetooth))                   any of the above, filtered by text
/// (Grep ((Session 47318657) [two words]))  bracket-quote multi-word filter text
/// ```
///
/// `Grep` wraps another query: it narrows which answers print without changing
/// which transcripts are read, and it composes, so nested `Grep`s all apply.
#[derive(NotaDecode, NotaEncode, Debug, Clone, PartialEq, Eq)]
pub enum Query {
    Latest,
    All,
    Session(String),
    File(String),
    Grep(Box<Query>, String),
}

impl Query {
    /// Decode one NOTA argument token into a query.
    pub fn parse(argument: &str) -> Result<Self> {
        Ok(NotaSource::new(argument).parse::<Query>()?)
    }

    /// Read every selected transcript, print each answer that passes every
    /// filter, and return how many answers were written.
    pub fn run(&self, project: &ProjectDirectory, writer: &mut impl Write) -> Result<usize> {
        let filters = self.filters();
        let mut total = 0;
        for path in self.transcripts(project)? {
            let transcript = Transcript::at(path);
            let mut written = 0;
            for answer in transcript.answers()? {
                if !filters.iter().all(|needle| answer.matches(needle)) {
                    continue;
                }
                answer.render(writer)?;
                written += 1;
            }
            if written > 0 {
                writeln!(writer, "# ^ {written} answer(s) in {}", transcript.name())?;
                writeln!(writer)?;
                total += written;
            }
        }
        Ok(total)
    }

    /// Every text filter this query imposes, outermost first. A query with no
    /// `Grep` wrapper imposes none, so every answer passes.
    fn filters(&self) -> Vec<&str> {
        match self {
            Query::Grep(inner, needle) => {
                let mut needles = vec![needle.as_str()];
                needles.extend(inner.filters());
                needles
            }
            _ => Vec::new(),
        }
    }

    /// The transcript files this query names, most-recent last. `Grep` is
    /// transparent here: it delegates to the query it wraps.
    fn transcripts(&self, project: &ProjectDirectory) -> Result<Vec<PathBuf>> {
        match self {
            Query::Latest => Ok(project
                .transcripts_by_age()?
                .into_iter()
                .next_back()
                .into_iter()
                .collect()),
            Query::All => project.transcripts_by_age(),
            Query::Session(fragment) => project.transcripts_matching(fragment),
            Query::File(path) => Ok(vec![PathBuf::from(path)]),
            Query::Grep(inner, _) => inner.transcripts(project),
        }
    }
}
