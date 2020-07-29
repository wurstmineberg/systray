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
            JsonElement status;
            try
            {
                people = GetJson(new Uri("https://wurstmineberg.de/api/v3/people.json")).GetProperty("people");
                status = GetJson(new Uri("https://wurstmineberg.de/api/v3/world/wurstmineberg/status.json"));
            }
            catch (HttpRequestException e)
            {
                //TODO change icon to an error icon
                ni.ContextMenuStrip = null;
                ni.Text = "error getting data";
                ni.ContextMenuStrip = new ContextMenus().FromException(e);
                return;
            }

            ni.Icon = Resources.wurstpick_white; //TODO use wurstpick_black if taskbar uses light theme
            ni.ContextMenuStrip = new ContextMenus().Create(people, status);
            if (!status.GetProperty("running").GetBoolean())
            {
                //TODO showIfOffline config
                ni.Visible = false;
            }
            else
	        {
                JsonElement list;
                if (!status.TryGetProperty("list", out list))
                {
                    list = JsonDocument.Parse("[]", new JsonDocumentOptions { }).RootElement;
                }
                if (list.GetArrayLength() == 0) //TODO && showIfEmpty config isn't set to true
                {
                    ni.Visible = false;
                }
                else
                {
                    ni.Visible = true;
                    if (list.GetArrayLength() == 1)
                    {
                        JsonElement uid = Enumerable.Single<JsonElement>(list.EnumerateArray());
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
                        ni.Text = $"{list.GetArrayLength()} players are online";
                    }
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
