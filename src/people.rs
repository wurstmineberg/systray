use {
    std::{
        collections::HashMap,
        fmt,
    },
    serde::Deserialize,
    serenity::all::UserId,
};

#[derive(Deserialize)]
#[serde(try_from = "u8")]
struct PeopleFileVersion;

impl PeopleFileVersion {
    const CURRENT: u8 = 3;
}

#[derive(Debug, thiserror::Error)]
#[error("people file returned from API has version {0} but this app only supports version {}", PeopleFileVersion::CURRENT)]
struct PeopleFileVersionError(u8);

impl TryFrom<u8> for PeopleFileVersion {
    type Error = PeopleFileVersionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == Self::CURRENT {
            Ok(Self)
        } else {
            Err(PeopleFileVersionError(value))
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct VersionedPeopleFile {
    #[allow(unused)] // only deserialized for the version check
    version: PeopleFileVersion,
    pub(crate) people: HashMap<Uid, Person>,
}

#[derive(Deserialize)]
pub(crate) struct Person {
    pub(crate) name: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(untagged)]
pub(crate) enum Uid {
    Snowflake(UserId),
    WmbId(String),
}

impl fmt::Display for Uid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Snowflake(snowflake) => snowflake.fmt(f),
            Self::WmbId(wmb_id) => wmb_id.fmt(f),
        }
    }
}
