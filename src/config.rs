use {
    std::{
        collections::HashMap,
        fs,
    },
    directories::BaseDirs,
    serde::Deserialize,
};

fn make_true() -> bool { true }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Config {
    #[serde(default = "make_true")]
    pub(crate) left_click_launch: bool,
    #[serde(default)]
    pub(crate) show_if_empty: bool,
    #[serde(default)]
    pub(crate) show_if_offline: bool,
    #[serde(default)]
    pub(crate) version_match: HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error(transparent)] Io(#[from] std::io::Error),
    #[error(transparent)] Json(#[from] serde_json::Error),
    #[error(transparent)] Wheel(#[from] wheel::Error),
    #[error("failed to find user folder")]
    BaseDirs,
}

impl Config {
    pub(crate) fn blocking_load() -> Result<Self, Error> {
        let path = BaseDirs::new().ok_or(Error::BaseDirs)?.data_dir().join("Wurstmineberg").join("config.json");
        Ok(if path.exists() {
            serde_json::from_str(&fs::read_to_string(path)?)?
        } else {
            Self::default()
        })
    }

    pub(crate) async fn load() -> Result<Self, Error> {
        let path = BaseDirs::new().ok_or(Error::BaseDirs)?.data_dir().join("Wurstmineberg").join("config.json");
        Ok(if wheel::fs::exists(&path).await? {
            wheel::fs::read_json(path).await?
        } else {
            Self::default()
        })
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            left_click_launch: true,
            show_if_empty: false,
            show_if_offline: false,
            version_match: HashMap::default(),
        }
    }
}
