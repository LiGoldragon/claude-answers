//! The authored Ethos map and checked recursive Datomic query anatomy.

use std::{fs, path::Path};

use claude_answers::Query;
use datomic::{Datomic, FaultProblem, Text, TextEdge};

struct EmptyManifest;

impl ethos_zero::Manifest for EmptyManifest {
    fn resolve(&self, _: &str) -> Option<ethos_zero::FileLocation> {
        None
    }
}

fn authored_map() -> ethos_zero::File {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("claude-answers.ethos");
    let source = fs::read_to_string(path).expect("read authored Ethos map");
    ethos_zero::FileReader::new(&EmptyManifest)
        .read(&source)
        .expect("the authored query map is valid Ethos")
}

#[test]
fn authored_ethos_declares_the_recursive_datomic_query() {
    let ethos_zero::File::Schema(schema) = authored_map() else {
        panic!("claude-answers owns a Schema map");
    };

    assert_eq!(schema.types.len(), 1);
    assert_eq!(schema.kinds.len(), 1);
}

#[test]
fn checked_d3_round_trips_the_authored_query_grammar() {
    for source in [
        "Latest",
        "All",
        "Session.47318657",
        "File./path/to.jsonl",
        "Grep.{All Bluetooth}",
        "Grep.{Session.47318657 “Bluetooth adapter”}",
        "Grep.{Grep.{Session.47318657 Bluetooth} adapter}",
    ] {
        let query = Text::<Query>::from(source)
            .embody()
            .expect("one authored Query value embodies at the typed text edge");
        assert_eq!(query.textualize().as_ref(), source);
    }
}

#[test]
fn checked_d3_refuses_wrong_query_shapes() {
    for (source, expected) in [
        ("Unknown", FaultProblem::Shape),
        ("Grep.All", FaultProblem::Shape),
        ("Grep.{All}", FaultProblem::Arity),
        ("Grep.{All Bluetooth extra}", FaultProblem::Arity),
        ("Grep.[All Bluetooth]", FaultProblem::Shape),
    ] {
        let fault = Text::<Query>::from(source)
            .embody()
            .expect_err("a non-Query shape is refused");
        assert!(matches!(
            (fault.problem, expected),
            (FaultProblem::Shape, FaultProblem::Shape) | (FaultProblem::Arity, FaultProblem::Arity)
        ));
    }
}
