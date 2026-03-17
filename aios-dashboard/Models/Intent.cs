namespace AiosDashboard.Models;

public class Intent
{
    [YamlDotNet.Serialization.YamlMember(Alias = "raw_text")]
    public string RawText { get; set; } = string.Empty;

    [YamlDotNet.Serialization.YamlMember(Alias = "target_capability")]
    public string? TargetCapability { get; set; }

    [YamlDotNet.Serialization.YamlMember(Alias = "parameters")]
    public Dictionary<string, string> Parameters { get; set; } = new();
}
