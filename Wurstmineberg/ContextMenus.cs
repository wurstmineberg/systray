using System;
using System.Diagnostics;
using System.Net.Http;
using System.Text.Json;
using System.Windows.Forms;

namespace Wurstmineberg
{
    class ContextMenus
    {
        public ContextMenuStrip Create(JsonElement people, JsonElement statuses)
        {
            // add the default menu options
            ContextMenuStrip menu = new ContextMenuStrip();
            ToolStripMenuItem item;

            foreach (JsonProperty world in statuses.EnumerateObject())
            {
                JsonElement status = world.Value;
                if (status.GetProperty("list").GetArrayLength() > 0) //TODO also show main world if offline
                {
                    // world name
                    item = new ToolStripMenuItem(world.Name);
                    item.Enabled = false;
                    menu.Items.Add(item);

                    // version link (TODO respect versionLink config)
                    item = new ToolStripMenuItem($"Version: {status.GetProperty("version").GetString()}");
                    item.Click += new EventHandler((sender, e) =>
                    {
                        var versionString = status.GetProperty("version").GetString();
                        Process.Start("explorer", $"https://minecraft.fandom.com/wiki/Java_Edition_{versionString}");
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
                    foreach (JsonElement uid in list.EnumerateArray())
                    {
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
                        item.Click += new EventHandler((sender, e) =>
                        {
                            Process.Start("explorer", $"https://wurstmineberg.de/people/{uidString}");
                        });
                        menu.Items.Add(item);
                    }

                    menu.Items.Add(new ToolStripSeparator());
                }
            }

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

        public ContextMenuStrip FromException(Exception e)
        {
            ContextMenuStrip menu = new ContextMenuStrip();
            ToolStripMenuItem item;

            item = new ToolStripMenuItem($"{e}");
            menu.Items.Add(item);

            return menu;
        }
    }
}
