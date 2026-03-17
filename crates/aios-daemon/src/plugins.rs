use aios_core::models::{ExecutionResult, Intent, SystemContext};
use aios_core::plugin::AiosNativeApp;

pub struct FileSystemApp;

impl AiosNativeApp for FileSystemApp {
    fn id(&self) -> &str {
        "core.fs"
    }

    fn describe_capabilities(&self) -> Vec<String> {
        vec![
            "List files in a directory [List(path)]".to_string(),
            "Read file contents [Read(path)]".to_string(),
            "Write string to file [Write(path, content)]".to_string(),
        ]
    }

    fn execute(&self, intent: &Intent, context: &SystemContext) -> ExecutionResult {
        // In a real system, this would actually manipulate the file system
        // and enforce context.permissions!
        let operation = intent.target_capability.as_deref().unwrap_or("Unknown");

        match operation {
            "List" => ExecutionResult {
                success: true,
                output: format!("Files in {}: ['README.md', 'crates/', 'Cargo.toml']", context.active_directory),
                error: None,
            },
            "Read" => ExecutionResult {
                success: true,
                output: format!("Simulated read from file context with user {}", context.user_id),
                error: None,
            },
            _ => ExecutionResult {
                success: false,
                output: "".to_string(),
                error: Some(format!("Unknown capability {} for {}", operation, self.id())),
            }
        }
    }
}
