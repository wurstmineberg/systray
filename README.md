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
* You can left-click on the icon to start Minecraft. This supports both the official Minecraft launcher and [Prism Launcher](https://prismlauncher.org/). For Prism Launcher to be detected, it must be available on the `PATH`. If Prism Launcher is installed via [Scoop](https://scoop.sh/), this should be the case by default.
* You can right-click on the icon to see the active worlds, their current versions (each with a link to the [Minecraft wiki](https://minecraft.fandom/) article about that version), as well as the full list of everyone who's online (with links to their Wurstmineberg profiles).

## Configuration

You can optionally configure the behavior of the app by creating a [JSON](https://json.org/) file at `%APPDATA%\Wurstmineberg\config.json`. All entries are optional:

* `leftClickLaunch`: Whether to open Minecraft when the systray icon is clicked. Defaults to `true`.
* `showIfEmpty`: If `false`, the plugin is hidden entirely if the main world is running but no players are online on any world. Defaults to `false`.
* `showIfOffline`: If `false`, the plugin is hidden entirely if the main world is not running and no players are online on any world. Defaults to `false`.
* `versionMatch`: An object mapping Minecraft launcher profile IDs to Wurstmineberg world names. Each launcher profile's selected Minecraft version will be kept in sync with the version running on that world.

# Building from source

If [pre-built binaries](https://github.com/fenhl/melt#installation) are not available for your system, if you would like to manage updates of the app using [`cargo-update`](https://crates.io/crates/cargo-update), or if you would like to run an unreleased version, follow these instructions:

1. Download and install [Visual Studio](https://visualstudio.microsoft.com/vs/) (the Community edition should work). On the “Workloads” screen of the installer, make sure “Desktop development with C++” is selected, and on the “Individual components” screen, select “VS 2022 C++ ARM64/ARM64EC build tools (Latest)” and “C++ Clang Compiler for Windows”. (Note that [Visual Studio Code](https://code.visualstudio.com/) is not the same thing as Visual Studio. You need VS, not VS Code.)
2. Add `C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\Llvm\x64\bin` to the `PATH`. For example, in PowerShell:
    ```pwsh
    $env:Path += ";C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Tools\Llvm\x64\bin"
    ```
3. Install Rust:
    * On Windows, download and run [rustup-init.exe](https://win.rustup.rs/) and follow its instructions.
    * On other platforms, please see [the Rust website](https://www.rust-lang.org/tools/install) for instructions.
4. Open a command line:
    * On Windows, right-click the start button, then click “Terminal”, “Windows PowerShell”, or “Command Prompt”.
    * On other platforms, look for an app named “Terminal” or similar.
5. In the command line, run the following command. Depending on your computer, this may take a while.
    ```pwsh
    cargo install --git=https://github.com/wurstmineberg/systray --branch=main
    ```
6. The exe will be available as `.cargo\bin\systray-wurstmineberg-status.exe` in your user folder. You can simply double-click the app to run it, no installation required. Nothing will happen if no one is online, but the icon will appear once there's someone in the main world.
7. To automatically start the app when you sign in, press <kbd>Win</kbd><kbd>R</kbd>, type `shell:startup`, and create a shortcut in the folder that appears.
8. To ensure that the icon is visible when someone's online and not hidden behind the “Show hidden icons” arrow:
    * On Windows 11: right-click on empty space in the taskbar, select “Taskbar settings”, click “Other system tray icons”, and enable the toggle for Wurstmineberg.
    * On Windows 10: right-click on that arrow, select “Taskbar settings”, click “Select which icons appear on the taskbar”, and enable the toggle for Wurstmineberg.

# Publishing a new version

1. Increment the `version` field in `Cargo.toml`. (Try to loosely follow [semantic versioning](https://semver.org/) with respect to the installation and usage sections above.)
2. Commit and push the changes
3. Run `cargo build --release --target=aarch64-pc-windows-msvc --target=x86_64-pc-windows-msvc --target=i686-pc-windows-msvc`
6. [Create a new release](https://github.com/wurstmineberg/systray/releases/new) with the tag and title matching the version, a summary of new features and notable bugfixes, and the following attachments:
    * `target\aarch64-pc-windows-msvc\release\systray-wurstmineberg-status.exe` as `wurstmineberg-arm.exe`
    * `target\x86_64-pc-windows-msvc\release\systray-wurstmineberg-status.exe` as `wurstmineberg-x64.exe`
    * `target\i686-pc-windows-msvc\release\systray-wurstmineberg-status.exe` as `wurstmineberg-x86.exe`
