//! The authored Ethos map and checked recursive Datomic query anatomy.

use std::{fs, path::Path};

use claude_answers::Query;
use datomic::{Datomic, FaultProblem, Text, TextEdge};
use ethos_zero::{Capability, TypeDeclaration, TypeExpression, VariantPayload};

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

    assert!(schema.imports.is_empty());
    assert!(schema.associations.is_empty());

    let [
        TypeDeclaration::Enum {
            name,
            generics,
            variants,
            ..
        },
    ] = schema.types.as_slice()
    else {
        panic!("the map owns exactly one Query enum declaration");
    };
    assert_eq!(name, "Query");
    assert!(generics.is_empty());

    let [latest, all, session, file, grep] = variants.as_slice() else {
        panic!("Query owns its complete five-variant anatomy");
    };
    assert_eq!(latest.name, "Latest");
    assert!(matches!(latest.payload, VariantPayload::Unit));
    assert_eq!(all.name, "All");
    assert!(matches!(all.payload, VariantPayload::Unit));
    assert_eq!(session.name, "Session");
    assert_reference_payload(&session.payload, "String");
    assert_eq!(file.name, "File");
    assert_reference_payload(&file.payload, "String");
    assert_eq!(grep.name, "Grep");
    let VariantPayload::InlineStruct(fields) = &grep.payload else {
        panic!("Grep carries its recursive selection and text needle");
    };
    let [selection, needle] = fields.as_slice() else {
        panic!("Grep has exactly its selection and needle portions");
    };
    assert_eq!(selection.name, "selection");
    assert_application(
        &selection.ty,
        "Box",
        &[TypeExpression::Reference("Query".to_owned())],
    );
    assert_eq!(needle.name, "needle");
    assert_reference(&needle.ty, "String");

    let [kind] = schema.kinds.as_slice() else {
        panic!("the map owns exactly the Datomic kind");
    };
    assert_eq!(kind.name, "Datomic");
    assert!(kind.generics.is_empty());
    assert!(kind.constraints.is_empty());
    assert!(kind.associated.is_empty());
    assert!(kind.methods.is_empty());
    let [embody, portion, textualize] = kind.capabilities.as_slice() else {
        panic!("Datomic owns embody, portion, and textualize methods");
    };
    assert_capability(
        embody,
        "embody",
        &application("Result", &[reference("Self"), reference("Fault")]),
    );
    assert_capability(portion, "portion", &reference("Portion"));
    assert_capability(
        textualize,
        "textualize",
        &application("Text", &[reference("Self")]),
    );
}

fn assert_capability(capability: &Capability, name: &str, output: &TypeExpression) {
    let Capability::Simple {
        name: actual,
        outputs,
    } = capability
    else {
        panic!("{name} is a simple Datomic capability");
    };
    assert_eq!(actual, name);
    assert_eq!(outputs.as_slice(), std::slice::from_ref(output));
}

fn assert_reference_payload(payload: &VariantPayload, expected: &str) {
    let VariantPayload::Type(expression) = payload else {
        panic!("{expected} is a data-carrying variant payload");
    };
    assert_reference(expression, expected);
}

fn assert_reference(expression: &TypeExpression, expected: &str) {
    assert_eq!(expression, &reference(expected));
}

fn assert_application(
    expression: &TypeExpression,
    constructor: &str,
    arguments: &[TypeExpression],
) {
    assert_eq!(expression, &application(constructor, arguments));
}

fn reference(name: &str) -> TypeExpression {
    TypeExpression::Reference(name.to_owned())
}

fn application(constructor: &str, arguments: &[TypeExpression]) -> TypeExpression {
    TypeExpression::Application {
        constructor: constructor.to_owned(),
        arguments: arguments.to_vec(),
    }
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
