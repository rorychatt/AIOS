using AiosDashboard.Models;
using System.Net.Sockets;
using System.Text;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace AiosDashboard.Connections;

public class AiosDaemonClient
{
    private readonly ISerializer _serializer;
    private readonly IDeserializer _deserializer;

    public AiosDaemonClient()
    {
        _serializer = new SerializerBuilder()
            .WithNamingConvention(UnderscoredNamingConvention.Instance)
            .Build();
            
        _deserializer = new DeserializerBuilder()
            .WithNamingConvention(UnderscoredNamingConvention.Instance)
            .IgnoreUnmatchedProperties()
            .Build();
    }

    public async Task<ExecutionResult> SendIntentAsync(string text)
    {
        try
        {
            using var client = new TcpClient("127.0.0.1", 9090);
            using var stream = client.GetStream();

            var intent = new Intent { RawText = text };
            var yamlRequest = _serializer.Serialize(intent);
            yamlRequest += "\n---\n";

            var requestBytes = Encoding.UTF8.GetBytes(yamlRequest);
            await stream.WriteAsync(requestBytes, 0, requestBytes.Length);

            using var reader = new StreamReader(stream, Encoding.UTF8);
            var buffer = new char[4096];
            var bytesRead = await reader.ReadAsync(buffer, 0, buffer.Length);
            
            var responseYaml = new string(buffer, 0, bytesRead);
            
            // Cleanup the document separators
            var cleanYaml = responseYaml.Split("---")[0].Trim();

            return _deserializer.Deserialize<ExecutionResult>(cleanYaml);
        }
        catch (Exception ex)
        {
            return new ExecutionResult
            {
                Success = false,
                Error = $"Failed to communicate with daemon: {ex.Message}"
            };
        }
    }
}
