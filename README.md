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

* `leftClickLaunch`: Whether to open the Minecraft launcher when the systray icon is clicked. Defaults to `true`.
* `versionMatch`: An object mapping Minecraft launcher profile IDs to Wurstmineberg world names. Each launcher profile's selected Minecraft version will be kept in sync with the version running on that world.

# Building from source

If you would like to run an unreleased version, you can build it yourself:

1. Clone this repository.
2. Open the file `Wurstmineberg.sln` in [Visual Studio 2019](https://visualstudio.microsoft.com/vs/).
3. Press <kbd>Ctrl</kbd><kbd>Shift</kbd><kbd>B</kbd> to build the program.
4. The exe will be available in `Wurstmineberg\bin\Debug\netcoreapp3.1\Wurstmineberg.exe`. You can run it from that directory, but you can't move it somewhere else because the other files in that directory are required.

# Publishing a new version

1. Increment the `Version` field in `Wurstmineberg\Wurstmineberg.csproj`. (Try to loosely follow [semantic versioning](https://semver.org/) with respect to the installation and usage sections above.)
2. Commit and push the changes
3. Run `dotnet publish -c Release -p:PublishSingleFile=true --self-contained true -p:IncludeNativeLibrariesForSelfExtract=true -r win-x64`
4. Run `dotnet publish -c Release -p:PublishSingleFile=true --self-contained true -p:IncludeNativeLibrariesForSelfExtract=true -r win-x86`
5. [Create a new release](https://github.com/wurstmineberg/systray/releases/new) with the tag and title matching the version, a summary of new features and notable bugfixes, and the attachments `Wurstmineberg\bin\Release\net5.0-windows\win-x64\publish\Wurstmineberg.exe` as `wurstmineberg-x64.exe` and `Wurstmineberg\bin\Release\net5.0-windows\win-x86\publish\Wurstmineberg.exe` as `wurstmineberg-x86.exe`
