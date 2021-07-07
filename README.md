This is a Windows notification area (systray) app that shows who is currently online on [Wurstmineberg](https://wurstmineberg.de/).

For an equivalent macOS app, see [bitbar-server-status](https://github.com/wurstmineberg/bitbar-server-status).

# Installation

1. Download the app: [64-bit](https://github.com/wurstmineberg/systray/releases/latest/download/wurstmineberg-x64.exe) • [32-bit](https://github.com/wurstmineberg/systray/releases/latest/download/wurstmineberg-x86.exe)
2. You can simply double-click the app to run it, no installation required. Nothing will happen if no one is online, but the icon will appear once there's someone in the main world.
3. To automatically start the app when you sign in, press <kbd>Win</kbd><kbd>R</kbd>, type `shell:startup`, and move the downloaded app into the folder that appears.
4. To ensure that the icon is visible when someone's online and not hidden behind the “Show hidden icons” arrow, right-click on that arrow, select “Taskbar settings”, click “Select which icons appear on the taskbar”, and enable the toggle for Wurstmineberg.

# Usage

* The icon only appears as long as someone is online on one of our worlds. You can hover over it to see how many people are online (and if it's only one player, their name).
* You can left-click on the icon to start Minecraft.
* You can right-click on the icon to see the active worlds, their current versions (each with a link to the [Minecraft wiki](https://minecraft.fandom/) article about that version), as well as the full list of everyone who's online (with links to their Wurstmineberg profiles).

## Configuration

You can optionally configure the behavior of the app by creating a [JSON](https://json.org/) file at `%APPDATA%\Wurstmineberg\config.json`. All entries are optional:

* `versionMatch`: An object mapping Minecraft launcher profile IDs to Wurstmineberg world names. Each launcher profile's selected Minecraft version will be kept in sync with the version running on that world.

# Building from source

If you would like to run an unreleased version, you can build it yourself:

1. Clone this repository.
2. Open the file `Wurstmineberg.sln` in [Visual Studio 2019](https://visualstudio.microsoft.com/vs/).
3. Press <kbd>Ctrl</kbd><kbd>Shift</kbd><kbd>B</kbd> to build the program.
4. The exe will be available in `Wurstmineberg\bin\Debug\netcoreapp3.1\Wurstmineberg.exe`. You can run it from that directory, but you can't move it somewhere else because the other files in that directory are required.

# Publishing a new version

1. In [Visual Studio 2019](https://visualstudio.microsoft.com/vs/), open the project properties (double-click the folder “Properties” in the Solution Explorer).
2. In the Package tab, increment the package version. (Try to loosely follow [semantic versioning](https://semver.org/) with respect to the installation and usage sections above.)
3. Save the properties, then commit and push the changes
4. Right-click the Wurstmineberg project (not the solution) in the Solution Explorer and select “Publish…”
5. If you haven't done this before, a dialog will ask you where you're publishing.
    1. Select “Folder”
    2. Choose `bin\Release\x64\` as the path
    3. Click “Finish”
    4. Edit the profile, changing the deployment mode to self-contained, the target runtime to win-x64, and enabling “Produce single file” and “Enable ReadyToRun compilation” in the file publish options
    5. Rename the profile from “FolderProfile” to “x64”
    6. Create a second publish profile with the same settings, except replacing `x64` with `x86` in the path, the target runtime, and the profile name
6. Select the x64 profile and click Publish. Wait until the console shows “Publish: 1 succeeded, 0 failed, 0 skipped”.
7. Repeat for the x86 profile.
8. [Create a new release](https://github.com/wurstmineberg/systray/releases/new) with the tag and title matching the version, a summary of new features and notable bugfixes, and the attachments `wurstmineberg\bin\Release\x64\Wurstmineberg.exe` as `wurstmineberg-x64.exe` and `wurstmineberg\bin\Release\x86\Wurstmineberg.exe` as `wurstmineberg-x86.exe`
