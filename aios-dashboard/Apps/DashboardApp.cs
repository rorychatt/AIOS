using AiosDashboard.Connections;

namespace AiosDashboard.Apps;

public record AiosChatMessage(string Author, string Text);

[App(icon: Icons.Cpu, title: "AIOS Dashboard")]
public class DashboardApp : ViewBase
{
    public override object? Build()
    {
        var daemonClient = UseService<AiosDaemonClient>();
        var inputState = UseState("");
        var messagesState = UseState<List<AiosChatMessage>>([
            new AiosChatMessage("System", "AIOS Dashboard connected. How can I help you today?")
        ]);
        var isProcessing = UseState(false);

        return Layout.Horizontal().Gap(4).Size(Size.Full()).Padding(4)
            | (Layout.Vertical().Gap(4).Width(Size.Units(64)) // Sidebar
                | Text.H2("AIOS Status")
                | new Card(
                    Layout.Vertical().Gap(2)
                    | Text.Block("Daemon").Bold()
                    | new Badge("Online").Success()
                    | new Separator()
                    | Text.Block("Loaded Plugins:").Bold()
                    | Text.Block("• core.fs").Color(Colors.Muted)
                    | Text.Block("• core.proc").Color(Colors.Muted)
                    | Text.Block("• core.net").Color(Colors.Muted)
                    | Text.Block("• core.llm.router").Color(Colors.Muted)
                )
            )
            | (Layout.Vertical().Gap(4).Size(Size.Full()) // Main content / Chat
                | Text.H2("Terminal Session")
                | new Chat(
                    messagesState.Value.Select(m => new Ivy.ChatMessage(m.Author == "User" ? ChatSender.User : ChatSender.Assistant, m.Text)).ToArray(),
                    onSend: async (e) => {
                        var text = e.Value;
                        if (string.IsNullOrWhiteSpace(text)) return;
                        
                        var newList = new List<AiosChatMessage>(messagesState.Value) {
                            new AiosChatMessage("User", text)
                        };
                        messagesState.Set(newList);
                        
                        isProcessing.Set(true);
                        var result = await daemonClient.SendIntentAsync(text);
                        
                        var updatedList = new List<AiosChatMessage>(messagesState.Value) {
                            new AiosChatMessage("AIOS", result.Success ? result.Output : $"Error: {result.Error}")
                        };
                        messagesState.Set(updatedList);
                        isProcessing.Set(false);
                    }
                ).Size(Size.Full())

            );

    }
}

