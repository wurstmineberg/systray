#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use {
    std::{
        cell::RefCell,
        collections::HashMap,
        io::prelude::*,
        path::PathBuf,
        process::Command,
        rc::Rc,
        sync::Arc,
        time::Duration,
    },
    directories::BaseDirs,
    itertools::Itertools as _,
    log_lock::*,
    native_windows_derive as nwd,
    native_windows_gui::{
        self as nwg,
        NativeUi as _,
    },
    open::that as open,
    serde::Deserialize,
    tokio::{
        io,
        runtime::Runtime,
        time::sleep,
    },
    wheel::{
        fs,
        traits::{
            CommandExt as _,
            IoResultExt as _,
            IsNetworkError,
            ReqwestResponseExt as _,
            SyncCommandOutputExt as _,
        },
    },
    crate::{
        config::Config,
        people::{
            Person,
            Uid,
        },
    },
};

mod config;
mod launcher;
mod people;

const MAIN_WORLD: &str = "wurstmineberg";

#[derive(Deserialize)]
struct WorldStatus {
    #[serde(default)]
    list: Vec<Uid>,
    running: bool,
    version: String,
}

type State = (HashMap<Uid, Person>, HashMap<String, WorldStatus>);

#[derive(Default, nwd::NwgUi)]
pub struct SystemTray {
    runtime: Option<Runtime>,
    config: Config,
    state: Arc<Mutex<Option<Result<State, Error>>>>,
    #[nwg_control]
    #[nwg_events(OnInit: [SystemTray::init])]
    window: nwg::MessageWindow,
    #[nwg_resource]
    embed: nwg::EmbedResource,
    #[nwg_control]
    #[nwg_events(OnNotice: [SystemTray::set_icon])]
    update_notice: nwg::Notice,
    event_handler: RefCell<Option<nwg::EventHandler>>,
    #[nwg_resource(source_embed: Some(&data.embed), source_embed_str: Some("LOGO_BLACK_16"))]
    logo_black_16: nwg::Icon,
    #[nwg_resource(source_embed: Some(&data.embed), source_embed_str: Some("LOGO_BLACK_32"))]
    logo_black_32: nwg::Icon,
    #[nwg_resource(source_embed: Some(&data.embed), source_embed_str: Some("LOGO_WHITE_16"))]
    logo_white_16: nwg::Icon,
    #[nwg_resource(source_embed: Some(&data.embed), source_embed_str: Some("LOGO_WHITE_32"))]
    logo_white_32: nwg::Icon,
    #[nwg_control(icon: Some(&data.logo_white_16), tip: Some("Wurstmineberg: Loading…"))]
    #[nwg_events(MousePressLeftUp: [SystemTray::click], OnContextMenu: [SystemTray::show_menu(RC_SELF)])]
    tray: nwg::TrayNotification,
    tray_menu: RefCell<nwg::Menu>,
    version_items: RefCell<Vec<(nwg::MenuItem, String)>>,
    user_items: RefCell<Vec<(nwg::MenuItem, Uid)>>,
    other_items: RefCell<Vec<nwg::MenuItem>>,
    sep: RefCell<nwg::MenuSeparator>,
    item_error: RefCell<nwg::MenuItem>,
    item_launch_minecraft: RefCell<nwg::MenuItem>,
    item_exit: RefCell<nwg::MenuItem>,
}

impl SystemTray {
    fn init(&self) {
        self.set_icon();
        self.runtime.as_ref().unwrap().spawn(maintain(self.state.clone(), self.update_notice.sender()));
    }

    fn set_icon(&self) {
        let is_light = registry::Hive::CurrentUser.open(r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize", registry::Security::QueryValue).ok()
            .and_then(|key| key.value("SystemUsesLightTheme").ok())
            .map_or(false, |data| matches!(data, registry::Data::U32(1)));
        let (visibility, tooltip) = lock!(@blocking lock = self.state; match *lock {
            Some(Ok((ref people, ref statuses))) => if statuses.values().any(|status| !status.list.is_empty()) || if statuses[MAIN_WORLD].running { self.config.show_if_empty } else { self.config.show_if_offline } {
                (true, if let Ok(uid) = statuses.values().flat_map(|world| &world.list).exactly_one() {
                    let person = people.get(uid).and_then(|person| person.name.clone()).unwrap_or_else(|| uid.to_string());
                    format!("{person} is online")
                } else {
                    format!("{} players are online", statuses.values().map(|world| world.list.len()).sum::<usize>())
                })
            } else {
                (false, String::default())
            },
            Some(Err(_)) => (true, format!("error getting data")),
            None => (true, format!("Wurstmineberg: Loading…")),
        });
        self.tray.set_visibility(visibility);
        self.tray.set_icon(match (is_light, nwg::scale_factor() >= 1.5) {
            (true, true) => &self.logo_black_32,
            (true, false) => &self.logo_black_16,
            (false, true) => &self.logo_white_32,
            (false, false) => &self.logo_white_16,
        });
        self.tray.set_tip(&tooltip);
    }

    fn show_menu(self: &Rc<Self>) {
        let (x, y) = nwg::GlobalCursor::position();
        let mut menu = nwg::Menu::default();
        nwg::Menu::builder()
            .popup(true)
            .parent(&self.window)
            .build(&mut menu).expect("failed to generate tray menu");
        self.user_items.borrow_mut().clear();
        self.other_items.borrow_mut().clear();
        let app = self.clone();
        if let Some(previous_event_handler) = self.event_handler.replace(Some(nwg::full_bind_event_handler(&self.window.handle, move |event, _, handle| match event {
            nwg::Event::OnMenuItemSelected => if handle == app.item_launch_minecraft.borrow().handle {
                app.launch_minecraft().expect("failed to launch Minecraft");
            } else if handle == app.item_exit.borrow().handle {
                app.exit();
            } else {
                for (item, version) in &*app.version_items.borrow() {
                    if handle == item.handle {
                        open(format!("https://minecraft.wiki/w/Java_Edition_{version}")).expect("failed to open wiki article");
                        return
                    }
                }
                for (item, uid) in &*app.user_items.borrow() {
                    if handle == item.handle {
                        open(format!("https://wurstmineberg.de/people/{uid}")).expect("failed to open user profile");
                        return
                    }
                }
            },
            _ => {}
        }))) {
            nwg::unbind_event_handler(&previous_event_handler);
        }
        lock!(@blocking lock = self.state; match *lock {
            Some(Ok((ref people, ref statuses))) => if statuses.values().any(|status| !status.list.is_empty()) || if statuses[MAIN_WORLD].running { self.config.show_if_empty } else { self.config.show_if_offline } {
                for (world_name, status) in statuses {
                    if (world_name == MAIN_WORLD && !status.running) || !status.list.is_empty() {
                        let mut item = nwg::MenuItem::default();
                        nwg::MenuItem::builder()
                            .text(world_name)
                            .disabled(true)
                            .parent(&menu)
                            .build(&mut item).expect("failed to generate tray menu");
                        self.other_items.borrow_mut().push(item);
                        //TODO respect versionLink config
                        let mut item = nwg::MenuItem::default();
                        nwg::MenuItem::builder()
                            .text(&format!("Version: {}", status.version))
                            .parent(&menu)
                            .build(&mut item).expect("failed to generate tray menu");
                        self.version_items.borrow_mut().push((item, status.version.clone()));
                        if !status.running {
                            let mut item = nwg::MenuItem::default();
                            nwg::MenuItem::builder()
                                .text("Server offline")
                                .disabled(true)
                                .parent(&menu)
                                .build(&mut item).expect("failed to generate tray menu");
                            self.other_items.borrow_mut().push(item);
                        }
                        for uid in &status.list {
                            let mut item = nwg::MenuItem::default();
                            nwg::MenuItem::builder()
                                .text(&people.get(uid).and_then(|person| person.name.clone()).unwrap_or_else(|| uid.to_string()))
                                .parent(&menu)
                                .build(&mut item).expect("failed to generate tray menu");
                            self.user_items.borrow_mut().push((item, uid.clone()));
                        }
                        nwg::MenuSeparator::builder()
                            .parent(&menu)
                            .build(&mut self.sep.borrow_mut()).expect("failed to generate tray menu");
                    }
                }
            },
            Some(Err(ref e)) => {
                nwg::MenuItem::builder()
                    .text(&e.to_string())
                    .disabled(true)
                    .parent(&menu)
                    .build(&mut self.item_error.borrow_mut()).expect("failed to generate tray menu");
                nwg::MenuSeparator::builder()
                    .parent(&menu)
                    .build(&mut self.sep.borrow_mut()).expect("failed to generate tray menu");
            }
            None => {}
        });
        nwg::MenuItem::builder()
            .text("Start Minecraft")
            .parent(&menu)
            .build(&mut self.item_launch_minecraft.borrow_mut()).expect("failed to generate tray menu");
        nwg::MenuItem::builder()
            .text("Exit")
            .parent(&menu)
            .build(&mut self.item_exit.borrow_mut()).expect("failed to generate tray menu");
        menu.popup(x, y);
        *self.tray_menu.borrow_mut() = menu;
    }

    fn click(&self) {
        if self.config.left_click_launch {
            self.launch_minecraft().expect("failed to launch Minecraft");
        }
    }

    fn launch_minecraft(&self) -> Result<(), LaunchError> {
        let game_version = if let Some(ref version_override) = self.config.ferium.version_override {
            Some(version_override.clone())
        } else {
            lock!(@blocking lock = self.state; {
                let (_, world_status) = lock.as_ref().ok_or(LaunchError::MissingState)?.as_ref().map_err(|e| LaunchError::State { display: e.to_string(), debug: format!("{e:?}") })?;
                world_status.get(MAIN_WORLD).map(|world_status| world_status.version.clone())
            })
        };
        let portablemc_work_dir = if let Some(ferium_profile) = self.config.ferium.profiles.get(MAIN_WORLD) {
            if let Some(ref game_version) = game_version {
                let previous_profile = self.config.ferium.command()
                    .arg("profile")
                    .release_create_no_window()
                    .check("ferium profile")?
                    .stdout;
                let mut previous_profile = String::from_utf8(previous_profile)?;
                previous_profile.truncate(previous_profile.find(" *").ok_or(LaunchError::FeriumProfileFormat)?);
                self.config.ferium.command()
                    .arg("profile")
                    .arg("switch")
                    .arg(ferium_profile)
                    .release_create_no_window()
                    .check("ferium profile switch")?;
                let current_profile = self.config.ferium.command()
                    .arg("profile")
                    .release_create_no_window()
                    .check("ferium profile")?
                    .stdout;
                self.config.ferium.command()
                    .arg("profile")
                    .arg("configure")
                    .arg("--game-version")
                    .arg(game_version)
                    .release_create_no_window()
                    .check("ferium profile configure --game-version")?;
                self.config.ferium.command()
                    .arg("upgrade")
                    .release_create_no_window()
                    .check("ferium upgrade")?;
                self.config.ferium.command()
                    .arg("profile")
                    .arg("switch")
                    .arg(previous_profile)
                    .release_create_no_window()
                    .check("ferium profile switch")?;
                current_profile.lines().find_map(|line| line.ok().and_then(|line| line.strip_prefix("        \r  Output directory:   ").map(|dir| {
                    let mut dir = PathBuf::from(dir);
                    dir.pop();
                    dir
                })))
            } else {
                None
            }
        } else {
            None
        };
        if let Some(ref portablemc_login) = self.config.portablemc.login {
            let mut cmd = Command::new("python");
            cmd.arg("-m");
            cmd.arg("portablemc");
            if let Some(work_dir) = portablemc_work_dir {
                cmd.arg("--work-dir");
                cmd.arg(work_dir);
            }
            cmd.arg("start");
            cmd.arg(format!("fabric:{}", game_version.unwrap_or_default()));
            cmd.arg("--server=wurstmineberg.de");
            cmd.arg("--login");
            cmd.arg(portablemc_login);
            cmd.release_create_no_window();
            cmd.spawn().at_command("python -m portablemc")?;
        } else {
            let mut prism_command = Command::new("prismlauncher");
            if let Some(ref instance) = self.config.prism_instance {
                prism_command.arg("--show");
                prism_command.arg(instance);
            }
            match prism_command.release_create_no_window().spawn() {
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::NotFound => match Command::new("C:\\Program Files (x86)\\Minecraft Launcher\\MinecraftLauncher.exe")
                    .release_create_no_window()
                    .spawn()
                {
                    Ok(_) => {}
                    Err(e) if e.kind() == io::ErrorKind::NotFound => {
                        Command::new("explorer")
                            .arg("shell:AppsFolder\\Microsoft.4297127D64EC6_8wekyb3d8bbwe!Minecraft")
                            .release_create_no_window()
                            .spawn().at_command("explorer shell:AppsFolder\\Microsoft.4297127D64EC6_8wekyb3d8bbwe!Minecraft")?;
                    }
                    Err(e) => return Err(e).at_command("C:\\Program Files (x86)\\Minecraft Launcher\\MinecraftLauncher.exe").map_err(LaunchError::from),
                },
                Err(e) => return Err(e).at_command("prismlauncher").map_err(LaunchError::from),
            }
        }
        Ok(())
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }
}

#[derive(Debug, thiserror::Error)]
enum LaunchError {
    #[error(transparent)] Utf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)] Wheel(#[from] wheel::Error),
    #[error("failed to parse `ferium profile` command output")]
    FeriumProfileFormat,
    #[error("missing server state")]
    MissingState,
    #[error("{display}")]
    State {
        display: String,
        debug: String,
    },
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)] Config(#[from] config::Error),
    #[error(transparent)] Json(#[from] serde_json::Error),
    #[error(transparent)] Reqwest(#[from] reqwest::Error),
    #[error(transparent)] Wheel(#[from] wheel::Error),
    #[error("failed to find user folder")]
    BaseDirs,
    #[error("no profile named “{0}” in launcher data")]
    UnknownLauncherProfile(String),
}

impl IsNetworkError for Error {
    fn is_network_error(&self) -> bool {
        match self {
            Self::Config(_) => false,
            Self::Json(_) => false,
            Self::Reqwest(e) => e.is_network_error(),
            Self::Wheel(e) => e.is_network_error(),
            Self::BaseDirs => false,
            Self::UnknownLauncherProfile(_) => false,
        }
    }
}

async fn get_state(http_client: &reqwest::Client) -> Result<State, Error> {
    let people = http_client.get("https://wurstmineberg.de/api/v3/people.json")
        .send().await?
        .detailed_error_for_status().await?
        .json_with_text_in_error::<people::VersionedPeopleFile>().await?
        .people;
    let statuses = http_client.get("https://wurstmineberg.de/api/v3/server/worlds.json")
        .query(&[("list", "1")])
        .send().await?
        .detailed_error_for_status().await?
        .json_with_text_in_error().await?;
    Ok((people, statuses))
}

async fn maintain_inner(state: Arc<Mutex<Option<Result<State, Error>>>>, update_notifier: nwg::NoticeSender) -> Result<(), Error> {
    let http_client = reqwest::Client::builder()
        .user_agent(concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION")))
        .timeout(Duration::from_secs(30))
        .use_rustls_tls()
        .https_only(true)
        .http2_prior_knowledge()
        .build()?;
    loop {
        let config = Config::load().await?; //TODO update config field of app? (make sure to keep overrides from CLI args)
        let new_state = match get_state(&http_client).await {
            Ok((people, mut statuses)) => {
                for status in statuses.values_mut() {
                    status.list.retain(|uid| !config.ignored_players.contains(uid));
                }
                if !config.version_match.is_empty() {
                    let base_dirs = BaseDirs::new().ok_or(Error::BaseDirs)?;
                    let mut launcher_data_path = base_dirs.data_dir().join(".minecraft").join("launcher_profiles_microsoft_store.json");
                    if !fs::exists(&launcher_data_path).await? {
                        launcher_data_path = base_dirs.data_dir().join(".minecraft").join("launcher_profiles.json");
                    }
                    let mut launcher_data = fs::read_json::<launcher::Data>(&launcher_data_path).await?;
                    let mut modified = false;
                    for (profile_id, world_name) in &config.version_match {
                        let launcher_profile = launcher_data.profiles.get_mut(profile_id).ok_or_else(|| Error::UnknownLauncherProfile(profile_id.clone()))?;
                        let world_version = &statuses[world_name].version;
                        if launcher_profile.last_version_id != *world_version {
                            launcher_profile.last_version_id = world_version.clone();
                            modified = true;
                        }
                    }
                    if modified {
                        let mut buf = serde_json::to_string_pretty(&launcher_data)?;
                        buf.push('\n');
                        fs::write(launcher_data_path, buf).await?;
                    }
                }
                Ok((people, statuses))
            }
            Err(e) if e.is_network_error() => Err(e),
            Err(e) => return Err(e),
        };
        lock!(state = state; *state = Some(new_state));
        update_notifier.notice();
        sleep(Duration::from_secs(45)).await;
    }
}

async fn maintain(state: Arc<Mutex<Option<Result<State, Error>>>>, update_notifier: nwg::NoticeSender) {
    if let Err(e) = maintain_inner(state, update_notifier).await {
        nwg::fatal_message(concat!(env!("CARGO_PKG_NAME"), ": fatal error"), &format!("{e}\nDebug info: ctx = maintain, {e:?}"))
    }
}

#[derive(Debug, thiserror::Error)]
enum MainError {
    #[error(transparent)] Config(#[from] config::Error),
    #[error(transparent)] Io(#[from] io::Error),
    #[error(transparent)] Nwg(#[from] nwg::NwgError),
}

#[derive(clap::Parser)]
struct Args {
    #[clap(long)]
    show_if_empty: bool,
}

impl Args {
    fn to_config(self) -> Result<Config, config::Error> {
        let Self { show_if_empty } = self;
        let mut config = Config::blocking_load()?;
        if show_if_empty {
            config.show_if_empty = true;
        }
        Ok(config)
    }
}

fn gui_main(args: Args) -> Result<(), MainError> {
    nwg::init()?;
    let app = SystemTray::build_ui(SystemTray {
        runtime: Some(Runtime::new()?),
        config: args.to_config()?,
        ..SystemTray::default()
    })?;
    nwg::dispatch_thread_events();
    drop(app);
    Ok(())
}

#[wheel::main]
fn main(args: Args) {
    if let Err(e) = gui_main(args) {
        nwg::fatal_message(concat!(env!("CARGO_PKG_NAME"), ": fatal error"), &format!("{e}\nDebug info: ctx = main, {e:?}"))
    }
}
