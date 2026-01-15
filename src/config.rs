use {
    std::{
        collections::HashMap,
        fs,
    },
    directories::BaseDirs,
    serde::Deserialize,
    tokio::process::Command,
    wheel::traits::IoResultExt as _,
    crate::Uid,
};

fn make_true() -> bool { true }

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) ignored_players: Vec<Uid>,
    #[serde(default = "make_true")]
    pub(crate) left_click_launch: bool,
    #[serde(default)]
    pub(crate) ferium: Ferium,
    #[serde(default)]
    pub(crate) portablemc: PortableMc,
    pub(crate) prism_instance: Option<String>,
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
        Ok(wheel::fs::read_json(path).await.missing_ok()?)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ignored_players: Vec::default(),
            left_click_launch: true,
            ferium: Ferium::default(),
            portablemc: PortableMc::default(),
            prism_instance: None,
            show_if_empty: false,
            show_if_offline: false,
            version_match: HashMap::default(),
        }
    }
}

/// Configuration for <https://github.com/gorilla-devs/ferium>
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Ferium {
    /// Maps Wurstmineberg world names to ferium profile names.
    #[serde(default)]
    pub(crate) profiles: HashMap<String, String>,
    pub(crate) version_override: Option<String>,
    pub(crate) github_token: Option<String>,
}

impl Ferium {
    pub(crate) fn command(&self) -> Command {
        let mut cmd = Command::new("ferium");
        if let Some(ref github_token) = self.github_token {
            cmd.arg("--github-token");
            cmd.arg(github_token);
        }
        cmd
    }
}

/// Configuration for <https://pypi.org/project/portablemc/>
#[derive(Debug, Default, Clone, Deserialize)]
pub(crate) struct PortableMc {
    /// Login email address. If this is specified, Minecraft will be launched using portablemc.
    pub(crate) login: Option<String>,
}
