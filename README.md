This is a Windows system tray app that shows who is currently online on [Wurstmineberg](https://wurstmineberg.de/).

For an equivalent macOS app, see [bitbar-server-status](https://github.com/wurstmineberg/bitbar-server-status).

# Installation

1. Press <kbd>Windows</kbd><kbd>I</kbd> to open the Settings app
2. Navigate to System → About
3. In the “Device specifications” section, check what the second half of the “System type” says, and download the appropriate file:

    * [ARM-based processor](https://github.com/wurstmineberg/systray/releases/latest/download/wurstmineberg-arm.exe)
    * [x86-based processor](https://github.com/wurstmineberg/systray/releases/latest/download/wurstmineberg-x86.exe)
    * [x64-based processor](https://github.com/wurstmineberg/systray/releases/latest/download/wurstmineberg-x64.exe)

    If your system type is not listed here, or if you would like to manage updates of the app using [`cargo-update`](https://crates.io/crates/cargo-update), follow [the instructions for building from source](#building-from-source).
4. You can simply double-click the app to run it, no installation required. Nothing will happen if no one is online, but the icon will appear once there's someone in the main world.
5. To automatically start the app when you sign in, press <kbd>Win</kbd><kbd>R</kbd>, type `shell:startup`, and move the downloaded app into the folder that appears.
6. To ensure that the icon is visible when someone's online and not hidden behind the “Show hidden icons” arrow:
    * On Windows 11: right-click on empty space in the taskbar, select “Taskbar settings”, click “Other system tray icons”, and enable the toggle for Wurstmineberg.
    * On Windows 10: right-click on that arrow, select “Taskbar settings”, click “Select which icons appear on the taskbar”, and enable the toggle for Wurstmineberg.

# Usage

* The icon only appears as long as someone is online on one of our worlds. You can hover over it to see how many people are online (and if it's only one player, their name).
* You can left-click on the icon to start Minecraft. This supports [portablemc](https://pypi.org/project/portablemc/), [Prism Launcher](https://prismlauncher.org/), and the official Minecraft launcher.
    * For portablemc to be used, the `.portablemc.login` [configuration](#configuration) entry must be specified.
    * For Prism Launcher to be used, it must be available on the `PATH`. If Prism Launcher is installed via [Scoop](https://scoop.sh/), this should be the case by default.
    * The official Minecraft launcher is the fallback if the conditions for using neither portablemc nor Prism Launcher are met. Both the new Microsoft Store launcher and the old launcher are supported.
* You can right-click on the icon to see the active worlds, their current versions (each with a link to the [Minecraft Wiki](https://minecraft.wiki/) article about that version), as well as the full list of everyone who's online (with links to their Wurstmineberg profiles).
* The app can be run from the command line with the `launch` subcommand to start Minecraft (same behavior as left-clicking on the system tray icon).

## Configuration

You can optionally configure the behavior of the app by creating a [JSON](https://json.org/) file at `%APPDATA%\Wurstmineberg\config.json`. All entries are optional:

* `leftClickLaunch`: Whether to open Minecraft when the systray icon is clicked. Defaults to `true`.
* `ignoredPlayers`: An array of Wurstmineberg IDs and/or Discord snowflakes of players who should not be listed. To ignore a player who has both a Wurstmineberg ID and a Discord snowflake, list the Discord snowflake.
* `prismInstance`: When using [Prism Launcher](https://prismlauncher.org/), directly navigate to the given instance ID's window instead of the launcher's main window. See also: [What is an instance ID, and where do I find it?](https://prismlauncher.org/wiki/getting-started/command-line-interface/#what-is-an-instance-id-and-where-do-i-find-it)
* `showIfEmpty`: If `false`, the plugin is hidden entirely if the main world is running but no players are online on any world. Defaults to `false`.
* `showIfOffline`: If `false`, the plugin is hidden entirely if the main world is not running and no players are online on any world. Defaults to `false`.
* `versionMatch`: An object mapping Minecraft launcher profile IDs to Wurstmineberg world names. Each launcher profile's selected Minecraft version will be kept in sync with the version running on that world.
* `ferium`: Optional configuration for [ferium](https://github.com/gorilla-devs/ferium):
    * `profiles`: An object mapping Wurstmineberg world names to ferium profile names. Each ferium profile's selected Minecraft version will be synced to the version running on that world on launch, and all mods will be updated.
    * `versionOverride`: A Minecraft version to use instead of syncing to the world's version.
    * `githubToken`: GitHub personal access token that will be passed to ferium.
* `portablemc`: Optional configuration for [portablemc](https://pypi.org/project/portablemc/):
    * `login`: Login email address. If this is specified, Minecraft will be launched using portablemc instead of trying Prism Launcher or the official Minecraft Launcher. Use `python -m portablemc login` to configure this before the first launch.

# Building from source

If [pre-built binaries](https://github.com/fenhl/melt#installation) are not available for your system, if you would like to manage updates of the app using [`cargo-update`](https://crates.io/crates/cargo-update), or if you would like to run an unreleased version, follow these instructions:

1. Install Rust:
    * On Windows, download and run [rustup-init.exe](https://win.rustup.rs/) and follow its instructions. If asked to install Visual C++ prerequisites, use the “Quick install via the Visual Studio Community installer” option. You can uncheck the option to launch Visual Studio when done.
    * On other platforms, please see [the Rust website](https://www.rust-lang.org/tools/install) for instructions.
2. Open a command line:
    * On Windows, right-click the start button, then click “Terminal”, “Windows PowerShell”, or “Command Prompt”.
    * On other platforms, look for an app named “Terminal” or similar.
3. In the command line, run the following command. Depending on your computer, this may take a while.
    ```pwsh
    cargo install --git=https://github.com/wurstmineberg/systray --branch=main
    ```
4. The exe will be available as `.cargo\bin\systray-wurstmineberg-status.exe` in your user folder. You can simply double-click the app to run it, no installation required. Nothing will happen if no one is online, but the icon will appear once there's someone in the main world.
5. To automatically start the app when you sign in, press <kbd>Win</kbd><kbd>R</kbd>, type `shell:startup`, and create a shortcut in the folder that appears.
6. To ensure that the icon is visible when someone's online and not hidden behind the “Show hidden icons” arrow:
    * On Windows 11: right-click on empty space in the taskbar, select “Taskbar settings”, click “Other system tray icons”, and enable the toggle for Wurstmineberg.
    * On Windows 10: right-click on that arrow, select “Taskbar settings”, click “Select which icons appear on the taskbar”, and enable the toggle for Wurstmineberg.

# Publishing a new version

1. Increment the `version` field in `Cargo.toml`. (Try to loosely follow [semantic versioning](https://semver.org/) with respect to the installation and usage sections above.)
2. Ensure the version in `Cargo.lock` is updated (run `cargo check` if not)
3. Commit and push the changes
4. Run `cargo +nightly -Zscript .\assets\release.rs`
