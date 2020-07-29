using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Net.Http;
using System.Text;
using System.Text.Json;
using System.Threading.Tasks;
using System.Windows.Forms;

namespace Wurstmineberg
{
    class ContextMenus
    {
        public ContextMenuStrip Create(JsonElement people, JsonElement status)
        {
            // add the default menu options
            ContextMenuStrip menu = new ContextMenuStrip();
            ToolStripMenuItem item;

            // version link (TODO respect versionLink config)
            item = new ToolStripMenuItem($"Version: {status.GetProperty("version").GetString()}");
            item.Click += new EventHandler((sender, e) => {
                Process.Start($"https://minecraft.gamepedia.com/Java_Edition_{status.GetProperty("version").GetString()}");
            });
            menu.Items.Add(item);

            if (!status.GetProperty("running").GetBoolean())
            {
                menu.Items.Add(new ToolStripMenuItem("Server offline"));
            }
            JsonElement list;
            if (!status.TryGetProperty("list", out list))
            {
                list = JsonDocument.Parse("[]", new JsonDocumentOptions { }).RootElement;
            }
            foreach (JsonElement uid in list.EnumerateArray()) {
                String uidString = uid.ToString();
                //TODO avatar
                JsonElement person;
                if (!people.TryGetProperty(uidString, out person))
                {
                    person = JsonDocument.Parse("{}", new JsonDocumentOptions { }).RootElement;
                }
                JsonElement displayNameJson;
                String displayName;
                if (person.TryGetProperty("name", out displayNameJson))
                {
                    displayName = displayNameJson.GetString();
                }
                else
                {
                    displayName = uidString;
                }
                //TODO color?
                item = new ToolStripMenuItem(displayName);
                item.Click += new EventHandler((sender, e) => {
                    Process.Start($"https://wurstmineberg.de/people/{uidString}");
                });
                menu.Items.Add(item);
            }

            menu.Items.Add(new ToolStripSeparator());

            // start Minecraft
            item = new ToolStripMenuItem("Start Minecraft");
            item.Click += new EventHandler((sender, e) => {
                Process.Start("C:\\Program Files (x86)\\Minecraft Launcher\\MinecraftLauncher.exe");
            });
            menu.Items.Add(item);

            // exit
            item = new ToolStripMenuItem("Exit");
            item.Click += new System.EventHandler((sender, e) => {
                Application.Exit();
            });
            menu.Items.Add(item);

            return menu;
        }

        public ContextMenuStrip FromException(HttpRequestException e)
        {
            ContextMenuStrip menu = new ContextMenuStrip();
            ToolStripMenuItem item;

            item = new ToolStripMenuItem($"{e}");
            menu.Items.Add(item);

            return menu;
        }
    }
}
