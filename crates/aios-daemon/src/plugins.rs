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
            "Create a new folder [CreateFolder(path)]".to_string(),
            "Delete a file or folder [Delete(path)]".to_string(),
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
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let safe_target = target_dir.trim_start_matches(|c| c == '/' || c == '\\');
                    
                let full_path = Path::new(&context.active_directory).join(safe_target);
                
                if let (Ok(canonical_full), Ok(canonical_dir)) = (full_path.canonicalize(), Path::new(&context.active_directory).canonicalize()) {
                    if !canonical_full.starts_with(&canonical_dir) {
                        return ExecutionResult {
                            success: false,
                            output: "".to_string(),
                            error: Some("Security Error: Path traversal outside active directory forbidden".to_string()),
                        };
                    }
                } else {
                    return ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some("Security Error: Invalid directory or does not exist".to_string()),
                    };
                }

                match fs::read_dir(&full_path) {
                    Ok(entries) => {
                        let mut files = Vec::new();
                        for entry in entries.flatten() {
                            if let Ok(name) = entry.file_name().into_string() {
                                let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
                                if is_dir {
                                    files.push(format!("{}/", name));
                                } else {
                                    files.push(name);
                                }
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

                let safe_target = target_file.trim_start_matches(|c| c == '/' || c == '\\');
                let full_path = Path::new(&context.active_directory).join(safe_target);
                
                // Allow reading if the path is valid and canonically starts with the active directory
                if let (Ok(canonical_full), Ok(canonical_dir)) = (full_path.canonicalize(), Path::new(&context.active_directory).canonicalize()) {
                    if !canonical_full.starts_with(&canonical_dir) {
                        return ExecutionResult {
                            success: false,
                            output: "".to_string(),
                            error: Some("Security Error: Path traversal outside active directory forbidden".to_string()),
                        };
                    }
                } else {
                    return ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some("Security Error: Invalid path or file does not exist for reading".to_string()),
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
            "Write" => {
                if !context.permissions.contains(&"fs.write".to_string()) {
                    return ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some(
                            "Permission Denied: Missing 'fs.write' in SystemContext"
                                .to_string(),
                        ),
                    };
                }

                let target_file = intent
                    .parameters
                    .get("path")
                    .map(|s| s.as_str())
                    .unwrap_or("");
                let content = intent
                    .parameters
                    .get("content")
                    .map(|s| s.as_str())
                    .unwrap_or("");

                if target_file.is_empty() {
                    return ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some("Missing 'path' parameter for Write capability".to_string()),
                    };
                }

                let safe_target = target_file.trim_start_matches(|c| c == '/' || c == '\\');
                let full_path = Path::new(&context.active_directory).join(safe_target);
                
                // We use canonicalize on the parent directory since the file might not exist yet
                if let (Some(parent), Ok(canonical_dir)) = (full_path.parent(), Path::new(&context.active_directory).canonicalize()) {
                    if let Ok(canonical_parent) = parent.canonicalize() {
                        if !canonical_parent.starts_with(&canonical_dir) {
                            return ExecutionResult {
                                success: false,
                                output: "".to_string(),
                                error: Some("Security Error: Path traversal outside active directory forbidden".to_string()),
                            };
                        }
                    } else {
                         return ExecutionResult {
                            success: false,
                            output: "".to_string(),
                            error: Some("Security Error: Target directory does not exist".to_string()),
                        };
                    }
                }

                match fs::write(&full_path, content) {
                    Ok(_) => ExecutionResult {
                        success: true,
                        output: format!("Successfully wrote 'path: {}' with content: '{}'", target_file, content),
                        error: None,
                    },
                    Err(e) => ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some(format!("Failed to write file: {}", e)),
                    },
                }
            }
            "CreateFolder" => {
                if !context.permissions.contains(&"fs.write".to_string()) {
                    return ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some(
                            "Permission Denied: Missing 'fs.write' in SystemContext"
                                .to_string(),
                        ),
                    };
                }

                let target_dir = intent
                    .parameters
                    .get("path")
                    .map(|s| s.as_str())
                    .unwrap_or("");

                if target_dir.is_empty() {
                    return ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some("Missing 'path' parameter for CreateFolder capability".to_string()),
                    };
                }

                let safe_target = target_dir.trim_start_matches(|c| c == '/' || c == '\\');
                let full_path = Path::new(&context.active_directory).join(safe_target);
                
                // We use canonicalize on the parent directory since the folder doesn't exist yet
                if let (Some(parent), Ok(canonical_dir)) = (full_path.parent(), Path::new(&context.active_directory).canonicalize()) {
                    if let Ok(canonical_parent) = parent.canonicalize() {
                        if !canonical_parent.starts_with(&canonical_dir) {
                            return ExecutionResult {
                                success: false,
                                output: "".to_string(),
                                error: Some("Security Error: Path traversal outside active directory forbidden".to_string()),
                            };
                        }
                    } else {
                         return ExecutionResult {
                            success: false,
                            output: "".to_string(),
                            error: Some("Security Error: Target parent directory does not exist".to_string()),
                        };
                    }
                }

                match fs::create_dir_all(&full_path) {
                    Ok(_) => ExecutionResult {
                        success: true,
                        output: format!("Successfully created folder: '{}'", target_dir),
                        error: None,
                    },
                    Err(e) => ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some(format!("Failed to create folder: {}", e)),
                    },
                }
            }
            "Delete" => {
                if !context.permissions.contains(&"fs.write".to_string()) {
                    return ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some(
                            "Permission Denied: Missing 'fs.write' in SystemContext"
                                .to_string(),
                        ),
                    };
                }

                let target = intent
                    .parameters
                    .get("path")
                    .map(|s| s.as_str())
                    .unwrap_or("");

                if target.is_empty() {
                    return ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some("Missing 'path' parameter for Delete capability".to_string()),
                    };
                }

                let safe_target = target.trim_start_matches(|c| c == '/' || c == '\\');
                let full_path = Path::new(&context.active_directory).join(safe_target);

                if let (Ok(canonical_full), Ok(canonical_dir)) = (
                    full_path.canonicalize(),
                    Path::new(&context.active_directory).canonicalize(),
                ) {
                    if !canonical_full.starts_with(&canonical_dir) {
                        return ExecutionResult {
                            success: false,
                            output: "".to_string(),
                            error: Some(
                                "Security Error: Path traversal outside active directory forbidden"
                                    .to_string(),
                            ),
                        };
                    }
                } else {
                    return ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some("Security Error: Invalid path or does not exist".to_string()),
                    };
                }

                let res = if full_path.is_dir() {
                    fs::remove_dir_all(&full_path)
                } else {
                    fs::remove_file(&full_path)
                };

                match res {
                    Ok(_) => ExecutionResult {
                        success: true,
                        output: format!("Successfully deleted: '{}'", target),
                        error: None,
                    },
                    Err(e) => ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some(format!("Failed to delete: {}", e)),
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
