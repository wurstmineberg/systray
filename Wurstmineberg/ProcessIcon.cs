using System;
using System.Diagnostics;
using System.IO;
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
            ni.Text = "Wurstmineberg"; //TODO show number of online users in tooltip?
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
            JsonElement people = GetJson(new Uri("https://wurstmineberg.de/api/v3/people.json"));
            JsonElement status = GetJson(new Uri("https://wurstmineberg.de/api/v3/world/wurstmineberg/status.json"));

            ni.Icon = Resources.wurstpick_white; //TODO use wurstpick_black if taskbar uses light theme
            ni.ContextMenuStrip = new ContextMenus().Create(people.GetProperty("people"), status);
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
                }
            }
        }

        private JsonElement GetJson(Uri url)
        {
            HttpClient client = new HttpClient();
            Task<HttpResponseMessage> responseTask = client.GetAsync(url);
            responseTask.Wait();
            Task<Stream> bodyTask = responseTask.Result.Content.ReadAsStreamAsync();
            bodyTask.Wait();
            return JsonDocument.Parse(bodyTask.Result, new JsonDocumentOptions { }).RootElement;
        }
    }
}
