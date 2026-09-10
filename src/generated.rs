#![allow(dead_code, non_camel_case_types, non_snake_case)]
#[derive(datom_codec::Datomizable, datom_codec::Compositional)]
pub struct Grep_Data {
    pub query: std::boxed::Box<Query>,
    pub string: String,
}
#[derive(datom_codec::Datomizable, datom_codec::Compositional)]
pub enum Query {
    Latest,
    All,
    Session(String),
    File(String),
    Grep(Grep_Data),
}
#[derive(datom_codec::Datomizable, datom_codec::Compositional)]
pub struct Answer {
    pub first_string: String,
    pub second_string: String,
    pub third_string: String,
}
