namespace AiosDashboard.Models;

public class ExecutionResult
{
    [YamlDotNet.Serialization.YamlMember(Alias = "success")]
    public bool Success { get; set; }

    [YamlDotNet.Serialization.YamlMember(Alias = "output")]
    public string Output { get; set; } = string.Empty;

    [YamlDotNet.Serialization.YamlMember(Alias = "error")]
    public string? Error { get; set; }
}
