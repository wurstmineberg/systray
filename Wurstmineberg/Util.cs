using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Text.Json;

namespace Wurstmineberg
{
    class Util {
        public static void LaunchMinecraft() {
            try {
                Process.Start("C:\\Program Files (x86)\\Minecraft Launcher\\MinecraftLauncher.exe");
            } catch (System.ComponentModel.Win32Exception) {
                Process.Start("explorer", "shell:AppsFolder\\Microsoft.4297127D64EC6_8wekyb3d8bbwe!Minecraft");
            }
        }

        public static Config ReadConfig() {
            try {
                return JsonSerializer.Deserialize<Config>(File.ReadAllText(Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "Wurstmineberg", "config.json")));
            } catch (Exception ex) when (ex is DirectoryNotFoundException || ex is FileNotFoundException) {
                return new Config();
            }
        }
    }
}
