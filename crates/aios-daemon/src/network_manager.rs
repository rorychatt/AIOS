use aios_core::models::{ExecutionResult, Intent, SystemContext};
use aios_core::plugin::AiosNativeApp;
use sysinfo::{NetworkExt, System, SystemExt};

pub struct NetworkManagerApp;

impl AiosNativeApp for NetworkManagerApp {
    fn id(&self) -> &str {
        "core.net"
    }

    fn describe_capabilities(&self) -> Vec<String> {
        vec!["View network interfaces and traffic [IfConfig]".to_string()]
    }

    fn execute(&self, intent: &Intent, context: &SystemContext) -> ExecutionResult {
        if !context.permissions.contains(&"net.read".to_string()) {
            return ExecutionResult {
                success: false,
                output: "".to_string(),
                error: Some("Permission Denied: Missing 'net.read' in SystemContext".to_string()),
            };
        }

        let operation = intent.target_capability.as_deref().unwrap_or("Unknown");

        match operation {
            "IfConfig" => {
                let mut sys = System::new_all();
                sys.refresh_networks();

                let mut net_list = Vec::new();
                for (interface_name, data) in sys.networks() {
                    net_list.push(format!(
                        "{}: In: {} B, Out: {} B",
                        interface_name,
                        data.total_received(),
                        data.total_transmitted()
                    ));
                }

                ExecutionResult {
                    success: true,
                    output: format!("Network Interfaces:\n{}", net_list.join("\n")),
                    error: None,
                }
            }
            _ => ExecutionResult {
                success: false,
                output: "".to_string(),
                error: Some(format!(
                    "Unknown capability {} for {}",
                    operation,
                    self.id()
                )),
            },
        }
    }
}
