//! The public Datomic query boundary and its read-only execution.

use std::fmt;
use std::io::Write;
use std::path::PathBuf;

use datomic::{Datomic, DatomicString, PortionBuilding, PortionViewing, Text, TextEdge};
use protos::{Portion, Separator, StructuralEnclosure};

use crate::error::Result;
use crate::transcript::{ProjectDirectory, Transcript};

/// One representable text portion carried by a [`Query`].
///
/// Query values are data, rather than a lossy command-line string. This type
/// preserves the outgoing Datomic portion after validation, so a constructed
/// query cannot emit an unrepresentable string.
pub struct QueryText(DatomicString);

impl QueryText {
    /// Validate a value for use in a Datomic query.
    pub fn new(
        value: impl Into<String>,
    ) -> std::result::Result<Self, datomic::UnrepresentableString> {
        Ok(Self(DatomicString::try_from(value.into())?))
    }
}

impl AsRef<str> for QueryText {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl Clone for QueryText {
    fn clone(&self) -> Self {
        Self::new(self.as_ref()).expect("an embodied query string remains representable")
    }
}

impl fmt::Debug for QueryText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("QueryText")
            .field(&self.as_ref())
            .finish()
    }
}

impl PartialEq for QueryText {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}

impl Eq for QueryText {}

impl Datomic for QueryText {
    fn embody(portion: &Portion) -> std::result::Result<Self, datomic::Fault> {
        Ok(Self(DatomicString::embody(portion)?))
    }

    fn portion(&self) -> Portion {
        self.0.portion()
    }
}

/// The `claude-answers` argument, as one typed Datomic value.
///
/// ```text
/// Latest                                   newest transcript in this project
/// All                                      every transcript in this project
/// Session.47318657                         transcripts whose file name holds the id
/// File./path/to.jsonl                      one explicit transcript file
/// Grep.{All Bluetooth}                     any selection, filtered by text
/// Grep.{Session.47318657 “two words”}      curly-quote multi-word filter text
/// ```
///
/// `Grep` wraps another query: it narrows which answers print without changing
/// which transcripts are read, and it composes, so nested `Grep`s all apply.
/// Its recursive anatomy is checked manually here because the final
/// data-library emitter cannot implement Datomic for `Box<Query>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Query {
    Latest,
    All,
    Session(QueryText),
    File(QueryText),
    Grep(Box<Query>, QueryText),
}

impl Query {
    /// Embody one prospective Datomic text value as a query.
    pub fn parse(argument: &str) -> Result<Self> {
        Ok(Text::<Self>::from(argument).embody()?)
    }

    /// Make a session selector from one representable file-name fragment.
    pub fn session(
        fragment: impl Into<String>,
    ) -> std::result::Result<Self, datomic::UnrepresentableString> {
        Ok(Self::Session(QueryText::new(fragment)?))
    }

    /// Make an explicit-file selector from one representable path.
    pub fn file(
        path: impl Into<String>,
    ) -> std::result::Result<Self, datomic::UnrepresentableString> {
        Ok(Self::File(QueryText::new(path)?))
    }

    /// Wrap a selection in one representable case-insensitive filter.
    pub fn grep(
        selection: Self,
        needle: impl Into<String>,
    ) -> std::result::Result<Self, datomic::UnrepresentableString> {
        Ok(Self::Grep(Box::new(selection), QueryText::new(needle)?))
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
            Self::Grep(inner, needle) => {
                let mut needles = vec![needle.as_ref()];
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
            Self::Session(fragment) => project.transcripts_matching(fragment.as_ref()),
            Self::File(path) => Ok(vec![PathBuf::from(path.as_ref())]),
            Self::Grep(inner, _) => inner.transcripts(project),
        }
    }
}

impl Datomic for Query {
    fn embody(portion: &Portion) -> std::result::Result<Self, datomic::Fault> {
        if PortionViewing::bare_symbol(portion) == Some("Latest") {
            return Ok(Self::Latest);
        }
        if PortionViewing::bare_symbol(portion) == Some("All") {
            return Ok(Self::All);
        }

        let Some(headed) = PortionViewing::headed(portion) else {
            return Err(PortionViewing::fault(portion, datomic::FaultProblem::Shape));
        };
        if headed.separator != Separator::Period {
            return Err(PortionViewing::fault(portion, datomic::FaultProblem::Shape));
        }

        match headed.head.as_ref() {
            "Session" => Ok(Self::Session(QueryText::embody(&headed.body)?)),
            "File" => Ok(Self::File(QueryText::embody(&headed.body)?)),
            "Grep" => {
                let Some(parts) =
                    PortionViewing::structural(headed.body.as_ref(), StructuralEnclosure::Braced)
                else {
                    return Err(PortionViewing::fault(
                        headed.body.as_ref(),
                        datomic::FaultProblem::Shape,
                    ));
                };
                let [selection, needle] = parts else {
                    return Err(PortionViewing::fault(
                        headed.body.as_ref(),
                        datomic::FaultProblem::Arity,
                    ));
                };
                Ok(Self::Grep(
                    Box::new(Self::embody(selection)?),
                    QueryText::embody(needle)?,
                ))
            }
            _ => Err(PortionViewing::fault(portion, datomic::FaultProblem::Shape)),
        }
    }

    fn portion(&self) -> Portion {
        match self {
            Self::Latest => PortionBuilding::bare("Latest"),
            Self::All => PortionBuilding::bare("All"),
            Self::Session(fragment) => {
                PortionBuilding::headed("Session", Separator::Period, fragment.portion())
            }
            Self::File(path) => PortionBuilding::headed("File", Separator::Period, path.portion()),
            Self::Grep(selection, needle) => PortionBuilding::headed(
                "Grep",
                Separator::Period,
                PortionBuilding::structural(
                    "",
                    StructuralEnclosure::Braced,
                    vec![selection.portion(), needle.portion()],
                ),
            ),
        }
    }
}
