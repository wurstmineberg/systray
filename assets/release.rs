#!/usr/bin/env -S cargo +nightly -Zscript
---
[dependencies]
cargo_metadata = "0.23"
directories = "6"
open = "5"
reqwest = { version = "0.13", default-features = false, features = ["charset", "gzip", "hickory-dns", "http2", "rustls-no-provider", "system-proxy", "zstd"] }
rustls = { version = "0.23", default-features = false, features = ["ring"] }
sysinfo = { version = "0.37", default-features = false, features = ["system"] }
tempfile = "3"
thiserror = "2"
tokio = { version = "1", features = ["process"] }
wheel = { git = "https://github.com/fenhl/wheel", features = ["github"] }
---

use {
    std::{
        env,
        io::prelude::*,
        process,
        time::Duration,
    },
    directories::UserDirs,
    open::that as open,
    sysinfo::ProcessesToUpdate,
    tokio::process::Command,
    wheel::{
        fs,
        github::Repo,
        traits::{
            AsyncCommandOutputExt as _,
            IoResultExt as _,
        },
    },
};

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)] CargoMetadata(#[from] cargo_metadata::Error),
    #[error(transparent)] Http(#[from] reqwest::Error),
    #[error(transparent)] InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),
    #[error(transparent)] Wheel(#[from] wheel::Error),
    #[error("aborting due to empty release notes")]
    EmptyReleaseNotes,
    #[error("user folder not found")]
    MissingHomeDir,
    #[error("systray-wurstmineberg-status package missing from Cargo.toml")]
    MissingPackage,
    #[error("failed to stop systray-wurstmineberg-status")]
    SysinfoKill(sysinfo::KillError),
}

impl From<sysinfo::KillError> for Error {
    fn from(e: sysinfo::KillError) -> Self {
        Self::SysinfoKill(e)
    }
}

impl wheel::CustomExit for Error {
    fn exit(self, cmd_name: &'static str) -> ! {
        match self {
            Self::Wheel(wheel::Error::CommandExit { name, output }) => {
                eprintln!("command `{name}` exited with {}", output.status);
                eprintln!();
                if let Ok(stdout) = std::str::from_utf8(&output.stdout) {
                    eprintln!("stdout:");
                    eprintln!("{stdout}");
                } else {
                    eprintln!("stdout: {:?}", output.stdout);
                }
                if let Ok(stderr) = std::str::from_utf8(&output.stderr) {
                    eprintln!("stderr:");
                    eprintln!("{stderr}");
                } else {
                    eprintln!("stderr: {:?}", output.stderr);
                }
                process::exit(output.status.code().unwrap_or(1))
            }
            e => {
                eprintln!("{cmd_name}: {e}");
                eprintln!("debug info: {e:?}");
                process::exit(1)
            }
        }
    }
}

#[wheel::main(custom_exit)]
async fn main() -> Result<(), Error> {
    let _ = rustls::crypto::ring::default_provider().install_default();
    Command::new("git").arg("push").check("git push").await?;
    Command::new("rustup").arg("update").arg("stable").check("rustup").await?;
    Command::new("cargo").arg("+stable").arg("build").arg("--release").arg("--target=aarch64-pc-windows-msvc").arg("--target=x86_64-pc-windows-msvc").arg("--target=i686-pc-windows-msvc").check("cargo build").await?;
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::AUTHORIZATION, reqwest::header::HeaderValue::from_str(&format!("token {}", fs::read_to_string("assets/release-token").await?))?);
    let http_client = reqwest::Client::builder()
        .user_agent(concat!("wurstmineberg-systray-release/", env!("CARGO_PKG_VERSION"), " (https://github.com/wurstmineberg/systray)"))
        .default_headers(headers)
        .timeout(Duration::from_secs(600))
        .http2_prior_knowledge()
        .use_rustls_tls()
        .https_only(true)
        .build()?;
    let mut notes_file = tempfile::Builder::default()
        .prefix("wmb-systray-release-notes-")
        .suffix(".md")
        .tempfile().at_unknown()?;
    let mut cmd;
    let cmd_name = if env::var("TERM_PROGRAM").as_deref() == Ok("vscode") {
        cmd = Command::new("code.cmd");
        cmd.arg("--wait");
        "code"
    } else if env::var_os("STY").is_none() && env::var_os("SSH_CLIENT").is_none() && env::var_os("SSH_TTY").is_none() {
        cmd = Command::new("C:\\Program Files\\Microsoft VS Code\\bin\\code.cmd");
        "code"
    } else {
        unimplemented!("cannot edit release notes")
    };
    cmd.arg(notes_file.path()).spawn().at_command(cmd_name)?.check(cmd_name).await?; // spawn before checking to avoid capturing stdio
    let mut notes = String::default();
    notes_file.read_to_string(&mut notes).at(notes_file)?;
    if notes.is_empty() { return Err(Error::EmptyReleaseNotes) }
    let version = cargo_metadata::MetadataCommand::new()
        .exec()?
        .packages
            .into_iter()
            .find(|package| &*package.name == "systray-wurstmineberg-status").ok_or(Error::MissingPackage)?
            .version;
    let repo = Repo::new("wurstmineberg", "systray");
    let release = repo.create_release(&http_client, format!("systray-wurstmineberg-status {version}"), format!("v{version}"), notes).await?;
    let mut system = sysinfo::System::default();
    system.refresh_processes(ProcessesToUpdate::All, true);
    for process in system.processes_by_exact_name("systray-wurstmineberg-status.exe".as_ref()) {
        process.kill_and_wait()?;
    }
    repo.release_attach(&http_client, &release, "wurstmineberg-arm.exe", "application/vnd.microsoft.portable-executable", fs::read("target/aarch64-pc-windows-msvc/release/systray-wurstmineberg-status.exe").await?).await?;
    repo.release_attach(&http_client, &release, "wurstmineberg-x64.exe", "application/vnd.microsoft.portable-executable", fs::read("target/x86_64-pc-windows-msvc/release/systray-wurstmineberg-status.exe").await?).await?;
    repo.release_attach(&http_client, &release, "wurstmineberg-x86.exe", "application/vnd.microsoft.portable-executable", fs::read("target/i686-pc-windows-msvc/release/systray-wurstmineberg-status.exe").await?).await?;
    repo.publish_release(&http_client, release).await?;
    Command::new("cargo").arg("+stable").arg("install-update").arg("--git").arg("systray-wurstmineberg-status").spawn().at("cargo install-update")?.check("cargo install-update").await?;
    let bin_path = UserDirs::new().ok_or(Error::MissingHomeDir)?.home_dir().join(".cargo").join("bin").join("systray-wurstmineberg-status");
    open(&bin_path).at(bin_path)?;
    Ok(())
}
