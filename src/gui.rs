use {
    std::{
        collections::HashMap,
        hash::Hash,
        io::prelude::*,
        iter,
        path::PathBuf,
        sync::Arc,
    },
    futures::stream::StreamExt as _,
    iced::{
        Size,
        Subscription,
        Task,
        widget::*,
        window::{
            self,
            icon,
        },
    },
    native_windows_gui as nwg,
    tokio::{
        io,
        process::Command,
        sync::{
            broadcast,
            mpsc,
        },
        task::JoinHandle,
    },
    tokio_stream::wrappers::{
        BroadcastStream,
        ReceiverStream,
    },
    wheel::traits::{
        AsyncCommandOutputExt as _,
        CommandExt as _,
        IoResultExt as _,
        SendResultExt as _,
    },
    crate::{
        MAIN_WORLD,
        State,
        config::Config,
    },
};

#[derive(Debug, thiserror::Error, wheel::FromArc)]
enum LaunchError {
    #[error(transparent)] Config(#[from] crate::config::Error),
    #[error(transparent)] Reqwest(#[from] reqwest::Error),
    #[error(transparent)] #[from_arc] State(#[from] Arc<crate::Error>),
    #[error(transparent)] Utf8(#[from] std::string::FromUtf8Error),
    #[error(transparent)] Wheel(#[from] wheel::Error),
    #[error("failed to parse `ferium profile` command output")]
    FeriumProfileFormat,
}

async fn launch_minecraft(config: Option<Config>, http_client: &reqwest::Client, state: Option<Result<State, Arc<crate::Error>>>, wait: bool, window: window::Id, tx: mpsc::Sender<Message>) -> Result<(), LaunchError> {
    let config = if let Some(config) = config {
        config
    } else {
        tx.send(Message::Progress(window, "loading config")).await.allow_unreceived();
        Config::load().await?
    };
    let game_version = if let Some(ref version_override) = config.ferium.version_override {
        Some(version_override.clone())
    } else {
        let (_, world_status) = if let Some(state) = state {
            state?
        } else {
            tx.send(Message::Progress(window, "getting server version")).await.allow_unreceived();
            crate::get_state(http_client).await?
        };
        world_status.get(MAIN_WORLD).and_then(|world_status| world_status.version.clone())
    };
    let portablemc_work_dir = if let Some(ferium_profile) = config.ferium.profiles.get(MAIN_WORLD) {
        if let Some(ref game_version) = game_version {
            tx.send(Message::Progress(window, "checking active Ferium profile")).await.allow_unreceived();
            let previous_profile = config.ferium.command()
                .arg("profile")
                .release_create_no_window()
                .check("ferium profile").await?
                .stdout;
            let mut previous_profile = String::from_utf8(previous_profile)?;
            previous_profile.truncate(previous_profile.find(" *").ok_or(LaunchError::FeriumProfileFormat)?);
            tx.send(Message::Progress(window, "switching Ferium profiles")).await.allow_unreceived();
            config.ferium.command()
                .arg("profile")
                .arg("switch")
                .arg(ferium_profile)
                .release_create_no_window()
                .kill_on_drop(true)
                .check("ferium profile switch").await?;
            tx.send(Message::Progress(window, "getting Ferium profile data")).await.allow_unreceived();
            let current_profile = config.ferium.command()
                .arg("profile")
                .release_create_no_window()
                .kill_on_drop(true)
                .check("ferium profile").await?
                .stdout;
            tx.send(Message::Progress(window, "setting game version")).await.allow_unreceived();
            config.ferium.command()
                .arg("profile")
                .arg("configure")
                .arg("--game-version")
                .arg(game_version)
                .release_create_no_window()
                .kill_on_drop(true)
                .check("ferium profile configure --game-version").await?;
            tx.send(Message::Progress(window, "updating mods")).await.allow_unreceived();
            config.ferium.command()
                .arg("upgrade")
                .release_create_no_window()
                .kill_on_drop(true)
                .check("ferium upgrade").await?;
            tx.send(Message::Progress(window, "restoring active Ferium profile")).await.allow_unreceived();
            config.ferium.command()
                .arg("profile")
                .arg("switch")
                .arg(previous_profile)
                .release_create_no_window()
                .kill_on_drop(true)
                .check("ferium profile switch").await?;
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
    if let Some(ref portablemc_uuid) = config.portablemc.uuid {
        let mut cmd = Command::new("portablemc");
        cmd.arg("start");
        if let Some(work_dir) = portablemc_work_dir {
            cmd.arg("--mc-dir");
            cmd.arg(work_dir);
        }
        cmd.arg("--auth");
        cmd.arg("--uuid");
        cmd.arg(portablemc_uuid.to_string());
        cmd.arg("--join-server=wurstmineberg.de");
        cmd.arg(format!("fabric:{}", game_version.unwrap_or_default()));
        cmd.release_create_no_window();
        cmd.kill_on_drop(true);
        let child = cmd.spawn().at_command("portablemc")?;
        if wait {
            tx.send(Message::Progress(window, "launching Minecraft via new portablemc")).await.allow_unreceived();
            child.check("portablemc").await?;
        }
    } else if let Some(ref portablemc_email) = config.portablemc.email {
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
        cmd.arg(portablemc_email);
        cmd.release_create_no_window();
        cmd.kill_on_drop(true);
        let child = cmd.spawn().at_command("python -m portablemc")?;
        if wait {
            tx.send(Message::Progress(window, "launching Minecraft via old portablemc")).await.allow_unreceived();
            child.check("python -m portablemc").await?;
        }
    } else {
        let mut prism_command = Command::new("prismlauncher");
        if let Some(ref instance) = config.prism_instance {
            prism_command.arg("--show");
            prism_command.arg(instance);
        }
        match prism_command.release_create_no_window().kill_on_drop(true).spawn() {
            Ok(child) => if wait {
                tx.send(Message::Progress(window, "launching Minecraft via Prism")).await.allow_unreceived();
                child.check("prismlauncher").await?;
            },
            Err(e) if e.kind() == io::ErrorKind::NotFound => match Command::new("C:\\Program Files (x86)\\Minecraft Launcher\\MinecraftLauncher.exe")
                .release_create_no_window()
                .kill_on_drop(true)
                .spawn()
            {
                Ok(child) => if wait {
                    tx.send(Message::Progress(window, "launching Minecraft via old launcher")).await.allow_unreceived();
                    child.check("C:\\Program Files (x86)\\Minecraft Launcher\\MinecraftLauncher.exe").await?;
                },
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    let child = Command::new("explorer")
                        .arg("shell:AppsFolder\\Microsoft.4297127D64EC6_8wekyb3d8bbwe!Minecraft")
                        .release_create_no_window()
                        .kill_on_drop(true)
                        .spawn().at_command("explorer shell:AppsFolder\\Microsoft.4297127D64EC6_8wekyb3d8bbwe!Minecraft")?;
                    if wait {
                        tx.send(Message::Progress(window, "launching Minecraft via new launcher")).await.allow_unreceived();
                        child.check("explorer shell:AppsFolder\\Microsoft.4297127D64EC6_8wekyb3d8bbwe!Minecraft").await?;
                    }
                }
                Err(e) => return Err(e).at_command("C:\\Program Files (x86)\\Minecraft Launcher\\MinecraftLauncher.exe").map_err(LaunchError::from),
            },
            Err(e) => return Err(e).at_command("prismlauncher").map_err(LaunchError::from),
        }
    }
    tx.send(Message::LaunchDone(window)).await.allow_unreceived();
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error(transparent)] BroadcastStream(#[from] tokio_stream::wrappers::errors::BroadcastStreamRecvError),
}

#[derive(Debug, Clone)]
pub(crate) enum Message {
    CloseRequested(window::Id),
    CommandError(Arc<Error>),
    Exit,
    HandleLauncherWindow {
        config: Option<Config>,
        state: Option<Result<State, Arc<crate::Error>>>,
        wait: bool,
        window: window::Id,
    },
    LaunchDone(window::Id),
    LaunchMinecraft {
        config: Option<Config>,
        state: Option<Result<State, Arc<crate::Error>>>,
        wait: bool,
    },
    Progress(window::Id, &'static str),
}

struct Gui {
    http_client: reqwest::Client,
    progress: HashMap<window::Id, &'static str>,
    task: Option<JoinHandle<()>>,
}

impl Gui {
    fn new(http_client: reqwest::Client) -> Self {
        Self {
            progress: HashMap::default(),
            task: None,
            http_client,
        }
    }

    fn title(&self, _: window::Id) -> String {
        format!("Launching Minecraft — Wurstmineberg")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::CloseRequested(window) => {
                if let Some(task) = self.task.take() {
                    task.abort();
                }
                window::close(window)
            }
            Message::CommandError(e) => nwg::fatal_message(concat!(env!("CARGO_PKG_NAME"), ": fatal error"), &format!("{e}\nDebug info: ctx = gui::CommandError, {e:?}")),
            Message::Exit => iced::exit(),
            Message::HandleLauncherWindow { config, state, wait, window } => {
                let http_client = self.http_client.clone();
                let (tx, rx) = mpsc::channel(32);
                self.task = Some(tokio::spawn(async move {
                    if let Err(e) = launch_minecraft(config, &http_client, state, wait, window, tx).await {
                        nwg::fatal_message(concat!(env!("CARGO_PKG_NAME"), ": fatal error"), &format!("{e}\nDebug info: ctx = gui::launch_minecraft, {e:?}"))
                    }
                }));
                Task::stream(ReceiverStream::new(rx))
            }
            Message::LaunchDone(window) => window::close(window),
            Message::LaunchMinecraft { config, state, wait } => window::open(window::Settings {
                size: Size { width: 512.0, height: 128.0 },
                icon: icon::from_file_data(include_bytes!("../assets/wurstpick.ico"), Some(::image::ImageFormat::Ico)).ok(),
                exit_on_close_request: false,
                ..window::Settings::default()
            }).1.map(move |window| Message::HandleLauncherWindow { config: config.clone(), state: state.clone(), wait, window }),
            Message::Progress(window, text) => {
                self.progress.insert(window, text);
                Task::none()
            }
        }
    }

    fn view(&self, window: window::Id) -> iced::Element<'_, Message> {
        Column::new()
            //TODO progress bar
            .push(self.progress.get(&window).copied().unwrap_or("initializing"))
            .spacing(8)
            .padding(8)
            .into()
    }
}

pub(crate) enum Args {
    Default {
        rx: broadcast::Receiver<Message>,
    },
    Launch {
        wait: bool,
    },
}

struct RxWrapper(broadcast::Receiver<Message>);

impl Hash for RxWrapper {
    fn hash<H: std::hash::Hasher>(&self, _: &mut H) {}
}

pub(crate) fn run(http_client: reqwest::Client, args: Args) -> iced::Result {
    fn theme(_: &Gui, _: window::Id) -> Option<Theme> { wheel::gui::theme() }

    let standalone = match args { Args::Default { .. } => None, Args::Launch { wait } => Some(wait) };
    iced::daemon(move || (
        Gui::new(http_client.clone()),
        if let Some(wait) = standalone { Task::done(Message::LaunchMinecraft { config: None, state: None, wait }) } else { Task::none() },
    ), Gui::update, Gui::view)
        .title(Gui::title)
        .subscription(move |_| Subscription::batch(
            if let Args::Default { rx } = &args {
                Some(Subscription::run_with(RxWrapper(rx.resubscribe()), |RxWrapper(rx)| BroadcastStream::new(rx.resubscribe()).map(|res| res.unwrap_or_else(|e| Message::CommandError(Arc::new(e.into()))))))
            } else {
                None
            }.into_iter()
            .chain(iter::once(iced::event::listen_with(|event, _, window| if let iced::Event::Window(window::Event::CloseRequested) = event {
                Some(Message::CloseRequested(window))
            } else {
                None
            })))
        ))
        .theme(theme)
        .default_font(iced::Font::with_name("DejaVu Sans"))
        .run()
}
