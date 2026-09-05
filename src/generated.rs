#![allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Query {
    Latest,
    All,
    Session(protos::Text),
    File(protos::Text),
    Grep(std::boxed::Box<Query>, protos::Text),
}
impl datom_codec::Datomic for Query {
    fn incorporate(site: datom_codec::Site<'_>) -> std::result::Result<Self, datom_codec::Fault> {
        let v = datom_codec::Sited::variant(site)?;
        match v.name {
            "Latest" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::Latest)
            }
            "All" => {
                datom_codec::Headed::nothing(v)?;
                std::result::Result::Ok(Self::All)
            }
            "Session" => std::result::Result::Ok(Self::Session(datom_codec::Carrying::body(v)?)),
            "File" => std::result::Result::Ok(Self::File(datom_codec::Carrying::body(v)?)),
            "Grep" => {
                let mut p = datom_codec::Headed::positions(v, 2)?;
                let p0: std::boxed::Box<Query> = datom_codec::Positional::position(&mut p)?;
                let p1: protos::Text = datom_codec::Positional::position(&mut p)?;
                std::result::Result::Ok(Self::Grep(p0, p1))
            }
            _ => std::result::Result::Err(datom_codec::Headed::reject(
                &v,
                datom_codec::Problem::UnknownVariant(
                    protos::Word::try_from(v.name).expect("variant name"),
                ),
            )),
        }
    }
}
impl protos::Conceivable<datom_codec::Datom> for Query {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(protos::Situated(
            protos::Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            match self {
                Self::Latest => datom_codec::Datom::Word(
                    datom_codec::DatomWord::try_from(
                        protos::Word::try_from("Latest").expect("static variant"),
                    )
                    .expect("stable variant"),
                ),
                Self::All => datom_codec::Datom::Word(
                    datom_codec::DatomWord::try_from(
                        protos::Word::try_from("All").expect("static variant"),
                    )
                    .expect("stable variant"),
                ),
                Self::Session(p0) => datom_codec::Datom::Variant(
                    protos::Symbol::try_from("Session").expect("static variant"),
                    std::boxed::Box::new(
                        protos::Conceivable::conceive(p0)
                            .expect("infallible datom ascent")
                            .1,
                    ),
                ),
                Self::File(p0) => datom_codec::Datom::Variant(
                    protos::Symbol::try_from("File").expect("static variant"),
                    std::boxed::Box::new(
                        protos::Conceivable::conceive(p0)
                            .expect("infallible datom ascent")
                            .1,
                    ),
                ),
                Self::Grep(p0, p1) => datom_codec::Datom::Variant(
                    protos::Symbol::try_from("Grep").expect("static variant"),
                    std::boxed::Box::new(datom_codec::Datom::Struct(vec![
                        protos::Conceivable::conceive(p0)
                            .expect("infallible datom ascent")
                            .1,
                        protos::Conceivable::conceive(p1)
                            .expect("infallible datom ascent")
                            .1,
                    ])),
                ),
            },
        ))
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answer(pub protos::Text, pub protos::Text, pub protos::Text);
impl datom_codec::Datomic for Answer {
    fn incorporate(site: datom_codec::Site<'_>) -> std::result::Result<Self, datom_codec::Fault> {
        let mut p = datom_codec::Sited::positions(site, 3)?;
        let p0: protos::Text = datom_codec::Positional::position(&mut p)?;
        let p1: protos::Text = datom_codec::Positional::position(&mut p)?;
        let p2: protos::Text = datom_codec::Positional::position(&mut p)?;
        std::result::Result::Ok(Self(p0, p1, p2))
    }
}
impl protos::Conceivable<datom_codec::Datom> for Answer {
    type Fault = std::convert::Infallible;
    fn conceive(&self) -> std::result::Result<protos::Situated<datom_codec::Datom>, Self::Fault> {
        std::result::Result::Ok(protos::Situated(
            protos::Situation {
                extent: protos::Extent(0, 0),
                children: vec![],
            },
            datom_codec::Datom::Struct(vec![
                protos::Conceivable::conceive(&self.0)
                    .expect("infallible datom ascent")
                    .1,
                protos::Conceivable::conceive(&self.1)
                    .expect("infallible datom ascent")
                    .1,
                protos::Conceivable::conceive(&self.2)
                    .expect("infallible datom ascent")
                    .1,
            ]),
        ))
    }
}
