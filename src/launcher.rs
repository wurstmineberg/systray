use {
    std::collections::BTreeMap,
    serde::{
        Deserialize,
        Serialize,
    },
    serde_json::Value as Json,
};

#[derive(Deserialize, Serialize)]
pub(crate) struct Data {
    pub(crate) profiles: BTreeMap<String, Profile>,
    #[serde(flatten)]
    _extra: BTreeMap<String, Json>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Profile {
    pub(crate) last_version_id: String,
    #[serde(flatten)]
    _extra: BTreeMap<String, Json>,
}
