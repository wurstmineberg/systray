using System;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Text.Json;
using System.Threading.Tasks;
using System.Windows.Forms;
using Wurstmineberg.Properties;

namespace Wurstmineberg
{
    class ProcessIcon : IDisposable
    {
        NotifyIcon ni;

        public ProcessIcon()
        {
            ni = new NotifyIcon();
        }

        public void Display()
        {
            ni.MouseClick += new MouseEventHandler(ni_MouseClick);
            ni.Text = "Wurstmineberg";
            Update();

            Timer timer = new Timer();
            timer.Interval = 45 * 1000; // 45 seconds
            timer.Tick += new EventHandler(timer_Tick);
            timer.Start();
        }

        public void Dispose()
        {
            ni.Dispose();
        }

        void ni_MouseClick(object sender, MouseEventArgs e)
        {
            if (e.Button == MouseButtons.Left)
            {
                Process.Start("C:\\Program Files (x86)\\Minecraft Launcher\\MinecraftLauncher.exe");
            }
        }

        private void timer_Tick(object sender, EventArgs e)
        {
            Update();
        }

        private void Update()
        {
            JsonElement people;
            JsonElement statuses;
            try
            {
                people = GetJson(new Uri("https://wurstmineberg.de/api/v3/people.json")).GetProperty("people");
                statuses = GetJson(new Uri("https://wurstmineberg.de/api/v3/server/worlds.json?list"));
            }
            catch (Exception e)
            {
                //TODO change icon to an error icon
                ni.ContextMenuStrip = null;
                ni.Visible = true;
                ni.Text = "error getting data";
                ni.ContextMenuStrip = new ContextMenus().FromException(e);
                return;
            }

            ni.Icon = Resources.wurstpick_white; //TODO use wurstpick_black if taskbar uses light theme
            ni.ContextMenuStrip = new ContextMenus().Create(people, statuses);
            if (statuses.EnumerateObject().All(property => property.Value.GetProperty("list").GetArrayLength() == 0)) //TODO respect showIfOffline and showIfEmpty configs
            {
                ni.Visible = false;
            }
            else
            {
                ni.Visible = true;
                int numOnline = statuses.EnumerateObject().Select((property, _) => property.Value.GetProperty("list").GetArrayLength()).Sum();
                if (numOnline == 1)
                {
                    JsonElement uid = Enumerable.Single<JsonElement>(statuses.EnumerateObject().SelectMany((property, _) => property.Value.GetProperty("list").EnumerateArray()));
                    String uidString = uid.ToString();
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
                    ni.Text = $"{displayName} is online";
                }
                else
                {
                    ni.Text = $"{numOnline} players are online";
                }
            }
        }

        private JsonElement GetJson(Uri url)
        {
            HttpClient client = new HttpClient();
            Task<HttpResponseMessage> responseTask = client.GetAsync(url);
            responseTask.Wait();
            HttpResponseMessage response = responseTask.Result;
            if (!response.IsSuccessStatusCode)
            {
                throw new HttpRequestException($"URL {url} returned status code {response.StatusCode} {response.ReasonPhrase}");
            }
            Task<Stream> bodyTask = response.Content.ReadAsStreamAsync();
            bodyTask.Wait();
            return JsonDocument.Parse(bodyTask.Result, new JsonDocumentOptions { }).RootElement;
        }
    }
}
