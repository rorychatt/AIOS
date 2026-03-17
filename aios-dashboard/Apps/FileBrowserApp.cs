using AiosDashboard.Connections;

namespace AiosDashboard.Apps;

[App(icon: Icons.Folder, title: "File Browser")]
public class FileBrowserApp : ViewBase
{
    public override object? Build()
    {
        var daemonClient = UseService<AiosDaemonClient>();
        var currentPath = UseState(".");
        var selectedFile = UseState<string?>(null);

        var filesQuery = UseQuery(
            key: currentPath.Value,
            fetcher: async (ct) => {
                var result = await daemonClient.SendIntentAsync("List files", "List", new Dictionary<string, string> { { "path", currentPath.Value } });
                if (!result.Success) throw new Exception(result.Error ?? "Unknown daemon error.");
                
                var output = result.Output;
                var start = output.IndexOf('[');
                var end = output.LastIndexOf(']');
                if (start != -1 && end != -1 && end > start) {
                    var arrayStr = output.Substring(start + 1, end - start - 1);
                    return arrayStr.Split(',')
                        .Select(s => s.Trim(' ', '"', '\''))
                        .Where(s => !string.IsNullOrEmpty(s))
                        .ToList();
                }
                return new List<string>();
            }
        );

        var fileContentQuery = UseQuery(
            () => selectedFile.Value,
            async (fileName, ct) => {
                var result = await daemonClient.SendIntentAsync("Read file", "Read", new Dictionary<string, string> { { "path", fileName } });
                if (!result.Success) throw new Exception(result.Error ?? "Error reading file.");
                return result.Output;
            }
        );

        return Layout.Horizontal().Gap(4).Size(Size.Full()).Padding(4)
            | (Layout.Vertical().Gap(4).Width(Size.Units(64))
                | Text.H2("File Explorer")
                | currentPath.ToTextInput().Placeholder("Path offset from daemon root...")
                | new Button("Refresh Directory").OnClick(() => filesQuery.Mutator.Revalidate())
                | (filesQuery.Loading ? Text.Block("Loading directory...") : null)
                | (filesQuery.Error != null ? Text.Block("Error: " + filesQuery.Error.Message).Color(Colors.Destructive) : null)
                | (filesQuery.Value != null ? 
                    Layout.Vertical().Gap(1)
                    | filesQuery.Value.Select(f => 
                        new Button(f).Variant(ButtonVariant.Outline).OnClick(() => selectedFile.Set(f))
                    )
                : null)
            )
            | (Layout.Vertical().Gap(4).Size(Size.Full())
                | Text.H2("File Viewer")
                | Text.Block(selectedFile.Value != null ? $"Viewing: {selectedFile.Value}" : "Select a file to view.").Muted()
                | (fileContentQuery.Loading ? Text.Block("Loading file content...") : null)
                | (fileContentQuery.Error != null ? Text.Block("Error: " + fileContentQuery.Error.Message).Color(Colors.Destructive) : null)
                | (fileContentQuery.Value != null ? 
                    new Card(Text.P(fileContentQuery.Value)).Size(Size.Full())
                : null)
            );
    }
}
