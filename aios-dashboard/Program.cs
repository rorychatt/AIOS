using AiosDashboard.Apps;
using AiosDashboard.Connections;
using Microsoft.Extensions.DependencyInjection;
using System.Globalization;
CultureInfo.DefaultThreadCurrentCulture = CultureInfo.DefaultThreadCurrentUICulture = new CultureInfo("en-US");
var server = new Server();
#if DEBUG
server.UseHotReload();
#endif
server.Services.AddSingleton<AiosDaemonClient>();

// Removed AddConnectionsFromAssembly since we explicitly added it just in case.
server.AddAppsFromAssembly();

var chromeSettings = new ChromeSettings().DefaultApp<DashboardApp>().UseTabs(preventDuplicates: true);
server.UseChrome(chromeSettings);
await server.RunAsync();
