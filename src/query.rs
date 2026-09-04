//! The public query boundary and its read-only execution.

use std::path::PathBuf;

use datomic::{Actualizable, Datom, Potential};

use crate::error::Result;
use crate::generated::{Answer, Query, QueryGrep};
use crate::transcript::{ProjectDirectory, Transcript};

/// Actualize one inline datom text value as a query.
pub fn parse(argument: &str) -> Result<Query> {
    let potential: Potential<Query, Datom> = argument.into();
    Ok(potential.actualize()?)
}

impl Query {
    /// Read every selected transcript, collect each answer that passes every
    /// filter, and return the matching answers.
    pub fn run(&self, project: &ProjectDirectory) -> Result<Vec<Answer>> {
        let filters = self.filters();
        let mut answers = Vec::new();
        for path in self.transcripts(project)? {
            let transcript = Transcript::at(path);
            for raw in transcript.answers()? {
                if !filters.iter().all(|needle| raw.matches(needle)) {
                    continue;
                }
                answers.push(Answer(
                    raw.question.clone(),
                    raw.option.clone(),
                    raw.notes.clone(),
                ));
            }
        }
        Ok(answers)
    }

    /// Every text filter this query imposes, outermost first. A query with no
    /// `Grep` wrapper imposes none, so every answer passes.
    fn filters(&self) -> Vec<&str> {
        match self {
            Self::Grep(QueryGrep(inner, needle)) => {
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
            Self::Latest => Ok(project
                .transcripts_by_age()?
                .into_iter()
                .next_back()
                .into_iter()
                .collect()),
            Self::All => project.transcripts_by_age(),
            Self::Session(fragment) => project.transcripts_matching(fragment),
            Self::File(path) => Ok(vec![PathBuf::from(path.as_str())]),
            Self::Grep(QueryGrep(inner, _)) => inner.transcripts(project),
        }
    }
}
