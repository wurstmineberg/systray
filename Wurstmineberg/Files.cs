using System.Collections.Generic;
using System.Text.Json;
using System.Text.Json.Serialization;

namespace Wurstmineberg {
    class Config {
        public Dictionary<string, string> versionMatch { get; set; }
    }

    class LauncherData {
        public Dictionary<string, LauncherProfile> profiles { get; set; }
        [JsonExtensionData]
        public Dictionary<string, JsonElement> ExtensionData { get; set; }
    }

    class LauncherProfile {
        public string lastVersionId { get; set; }
        [JsonExtensionData]
        public Dictionary<string, JsonElement> ExtensionData { get; set; }
    }
}
