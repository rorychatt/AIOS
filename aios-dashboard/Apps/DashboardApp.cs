using AiosDashboard.Connections;

namespace AiosDashboard.Apps;

public record ChatMessage(string Role, string Content);

[App(icon: Icons.Cpu, title: "AIOS Dashboard")]
public class DashboardApp : ViewBase
{
    public override object? Build()
    {
        var daemonClient = UseService<AiosDaemonClient>();
        var inputState = UseState("");
        var messagesState = UseState<List<ChatMessage>>([
            new ChatMessage("System", "AIOS Dashboard connected. How can I help you today?")
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
                | (new Card(
                    Layout.Vertical().Gap(4).Size(Size.Full())
                    | (Layout.Vertical().Gap(4).Size(Size.Full()).Align(Align.TopLeft)
                        | messagesState.Value.Select(m => 
                            Layout.Horizontal()
                                .Width(Size.Full())
                                .Align(m.Role == "User" ? Align.Right : Align.Left)
                            | new Callout(m.Content)
                                .Title(m.Role)
                        )
                    )
                    | (Layout.Horizontal().Gap(2).Align(Align.BottomCenter).Width(Size.Full())
                        | inputState.ToTextInput().Placeholder("Type a command or ask a question...")
                        | new Button("Send").Primary().Disabled(isProcessing.Value).OnClick(async () => {
                            if (string.IsNullOrWhiteSpace(inputState.Value)) return;
                            
                            var userText = inputState.Value;
                            inputState.Set("");
                            
                            var newList = new List<ChatMessage>(messagesState.Value) {
                                new ChatMessage("User", userText)
                            };
                            messagesState.Set(newList);
                            
                            isProcessing.Set(true);
                            
                            var result = await daemonClient.SendIntentAsync(userText);
                            
                            var updatedList = new List<ChatMessage>(messagesState.Value) {
                                new ChatMessage("AIOS", result.Success ? result.Output : $"Error: {result.Error}")
                            };
                            messagesState.Set(updatedList);
                            
                            isProcessing.Set(false);
                        })
                    )
                ).Size(Size.Full()))
            );
    }
}
