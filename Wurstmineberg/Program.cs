using System;
using System.Windows.Forms;

namespace Wurstmineberg {
    static class Program {
        /// <summary>
        /// The main entry point for the application.
        /// </summary>
        [STAThread]
        static void Main() {
            Application.SetHighDpiMode(HighDpiMode.SystemAware);
            Application.EnableVisualStyles();
            Application.SetCompatibleTextRenderingDefault(false);

            // Show the system tray icon.
            using (ProcessIcon pi = new ProcessIcon()) {
                pi.Display();
                Application.Run();
            }
        }
    }
}
