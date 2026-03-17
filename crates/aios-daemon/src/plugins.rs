use aios_core::models::{ExecutionResult, Intent, SystemContext};
use aios_core::plugin::AiosNativeApp;
use std::fs;
use std::path::Path;

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
        let operation = intent.target_capability.as_deref().unwrap_or("Unknown");

        match operation {
            "List" => {
                if !context.permissions.contains(&"fs.read".to_string()) {
                    return ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some(
                            "Permission Denied: Missing 'fs.read' in SystemContext".to_string(),
                        ),
                    };
                }

                let target_dir = intent
                    .parameters
                    .get("path")
                    .unwrap_or(&context.active_directory);
                match fs::read_dir(target_dir) {
                    Ok(entries) => {
                        let mut files = Vec::new();
                        for entry in entries.flatten() {
                            if let Ok(name) = entry.file_name().into_string() {
                                files.push(name);
                            }
                        }
                        ExecutionResult {
                            success: true,
                            output: format!("Files in {}: {:?}", target_dir, files),
                            error: None,
                        }
                    }
                    Err(e) => ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some(format!("Failed to list directory: {}", e)),
                    },
                }
            }
            "Read" => {
                if !context.permissions.contains(&"fs.read".to_string()) {
                    return ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some(
                            "Permission Denied: Missing 'fs.read' in SystemContext".to_string(),
                        ),
                    };
                }

                let target_file = intent
                    .parameters
                    .get("path")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                if target_file.is_empty() {
                    return ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some("Missing 'path' parameter for Read capability".to_string()),
                    };
                }

                let full_path = Path::new(&context.active_directory).join(target_file);
                // Basic directory traversal protection
                if !full_path.starts_with(&context.active_directory) {
                    return ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some(
                            "Security Error: Path traversal outside active directory forbidden"
                                .to_string(),
                        ),
                    };
                }

                match fs::read_to_string(&full_path) {
                    Ok(content) => ExecutionResult {
                        success: true,
                        output: content,
                        error: None,
                    },
                    Err(e) => ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some(format!("Failed to read file: {}", e)),
                    },
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
