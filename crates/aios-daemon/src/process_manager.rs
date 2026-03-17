use aios_core::models::{ExecutionResult, Intent, SystemContext};
use aios_core::plugin::AiosNativeApp;
use sysinfo::{ProcessExt, System, SystemExt};

pub struct ProcessManagerApp;

impl AiosNativeApp for ProcessManagerApp {
    fn id(&self) -> &str {
        "core.proc"
    }

    fn describe_capabilities(&self) -> Vec<String> {
        vec![
            "List running processes [Ps]".to_string(),
            "Kill a process by PID [Kill(pid)]".to_string(),
        ]
    }

    fn execute(&self, intent: &Intent, context: &SystemContext) -> ExecutionResult {
        if !context.permissions.contains(&"proc.manage".to_string()) {
            return ExecutionResult {
                success: false,
                output: "".to_string(),
                error: Some(
                    "Permission Denied: Missing 'proc.manage' in SystemContext".to_string(),
                ),
            };
        }

        let operation = intent.target_capability.as_deref().unwrap_or("Unknown");

        match operation {
            "Ps" => {
                let mut sys = System::new_all();
                sys.refresh_all();

                let mut proc_list = Vec::new();
                for (pid, process) in sys.processes() {
                    proc_list.push(format!("[{}] {}", pid, process.name()));
                }

                ExecutionResult {
                    success: true,
                    output: format!("Processes:\n{}", proc_list.join("\n")),
                    error: None,
                }
            }
            "Kill" => {
                let pid_str = intent
                    .parameters
                    .get("pid")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                if let Ok(pid) = pid_str.parse::<sysinfo::Pid>() {
                    let sys = System::new_all();
                    if let Some(process) = sys.process(pid) {
                        let success = process.kill();
                        ExecutionResult {
                            success,
                            output: if success {
                                format!("Killed PID {}", pid)
                            } else {
                                "".to_string()
                            },
                            error: if !success {
                                Some(format!("Failed to kill PID {}", pid))
                            } else {
                                None
                            },
                        }
                    } else {
                        ExecutionResult {
                            success: false,
                            output: "".to_string(),
                            error: Some(format!("PID {} not found", pid)),
                        }
                    }
                } else {
                    ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some("Valid 'pid' parameter required".to_string()),
                    }
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
