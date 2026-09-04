#![allow(dead_code)]
pub struct QueryGrep(pub Box<Query>, pub protos::Text);
pub enum Query {
    Latest,
    All,
    Session(protos::Text),
    File(protos::Text),
    Grep(QueryGrep),
}
impl datomic::Corporal<datomic::Datom> for Query {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Bare(s) if s == stringify!(Latest) => Ok(Self::Latest),
            datomic::Datom::Bare(s) if s == stringify!(All) => Ok(Self::All),
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                if head == stringify!(Session) =>
            {
                Ok(Self::Session(<protos::Text as datomic::Corporal<
                    datomic::Datom,
                >>::incorporate(*body)?))
            }
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                if head == stringify!(File) =>
            {
                Ok(Self::File(<protos::Text as datomic::Corporal<
                    datomic::Datom,
                >>::incorporate(*body)?))
            }
            datomic::Datom::Variant(head, protos::Separator::Period, Some(body))
                if head == stringify!(Grep) =>
            {
                Ok(Self::Grep(<QueryGrep as datomic::Corporal<
                    datomic::Datom,
                >>::incorporate(*body)?))
            }
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Variant, other),
            )),
        }
    }
}
impl datomic::Datomic for Query {
    fn datomize(&self) -> datomic::Datom {
        match self {
            Self::Latest => datomic::Datom::Bare(stringify!(Latest).to_owned()),
            Self::All => datomic::Datom::Bare(stringify!(All).to_owned()),
            Self::Session(value) => datomic::Datom::Variant(
                stringify!(Session).to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(value))),
            ),
            Self::File(value) => datomic::Datom::Variant(
                stringify!(File).to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(value))),
            ),
            Self::Grep(value) => datomic::Datom::Variant(
                stringify!(Grep).to_owned(),
                protos::Separator::Period,
                Some(Box::new(datomic::Datomic::datomize(value))),
            ),
        }
    }
}
impl datomic::Corporal<datomic::Datom> for QueryGrep {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Struct(fields) if fields.len() == 2usize => {
                let mut iter = fields.into_iter();
                Ok(Self(
                    <Box<Query> as datomic::Corporal<datomic::Datom>>::incorporate(
                        iter.next().unwrap(),
                    )?,
                    <protos::Text as datomic::Corporal<datomic::Datom>>::incorporate(
                        iter.next().unwrap(),
                    )?,
                ))
            }
            datomic::Datom::Struct(fields) => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Arity(2i64, fields.len() as i64),
            )),
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Struct, other),
            )),
        }
    }
}
impl datomic::Datomic for QueryGrep {
    fn datomize(&self) -> datomic::Datom {
        datomic::Datom::Struct(vec![
            datomic::Datomic::datomize(&self.0),
            datomic::Datomic::datomize(&self.1),
        ])
    }
}
pub struct Answer(pub protos::Text, pub protos::Text, pub protos::Text);
impl datomic::Corporal<datomic::Datom> for Answer {
    type Fault = datomic::Fault;
    fn incorporate(concept: datomic::Datom) -> std::result::Result<Self, datomic::Fault> {
        match concept {
            datomic::Datom::Struct(fields) if fields.len() == 3usize => {
                let mut iter = fields.into_iter();
                Ok(Self(
                    <protos::Text as datomic::Corporal<datomic::Datom>>::incorporate(
                        iter.next().unwrap(),
                    )?,
                    <protos::Text as datomic::Corporal<datomic::Datom>>::incorporate(
                        iter.next().unwrap(),
                    )?,
                    <protos::Text as datomic::Corporal<datomic::Datom>>::incorporate(
                        iter.next().unwrap(),
                    )?,
                ))
            }
            datomic::Datom::Struct(fields) => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Arity(3i64, fields.len() as i64),
            )),
            other => Err(datomic::Fault::Corporal(
                vec![],
                datomic::Problem::Shape(datomic::Expected::Struct, other),
            )),
        }
    }
}
impl datomic::Datomic for Answer {
    fn datomize(&self) -> datomic::Datom {
        datomic::Datom::Struct(vec![
            datomic::Datomic::datomize(&self.0),
            datomic::Datomic::datomize(&self.1),
            datomic::Datomic::datomize(&self.2),
        ])
    }
}
