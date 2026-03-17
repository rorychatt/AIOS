use aios_core::models::{ExecutionResult, Intent, SystemContext};
use aios_core::plugin::AiosNativeApp;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub struct LlmRouterApp;

impl AiosNativeApp for LlmRouterApp {
    fn id(&self) -> &str {
        "core.llm.router"
    }

    fn describe_capabilities(&self) -> Vec<String> {
        vec!["Route intent to OpenAI for reasoning [Route(intent_text)]".to_string()]
    }

    fn execute(&self, intent: &Intent, context: &SystemContext) -> ExecutionResult {
        let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3".to_string());
        let user_prompt = intent
            .parameters
            .get("intent_text")
            .unwrap_or(&intent.raw_text);

        let mut messages = vec![
            serde_json::json!({
                "role": "system",
                "content": "You are AIOS, the conversational AI-first Operating System kernel. The user is talking to you via a terminal. You can answer their questions naturally. If you need to perform actions on the OS itself to answer their question, output the exact CLI command to run inside a `[COMMAND]` block, like: `[COMMAND] aios-cli fs list . [/COMMAND]`. Available commands: \
                `aios-cli fs list <path>`, \
                `aios-cli fs read <file>`, \
                `aios-cli fs write <path> <content>` (use this BOTH for creating new files and modifying existing ones), \
                `aios-cli fs create-folder <path>`, \
                `aios-cli fs delete <path>`, \
                `aios-cli proc ps`, \
                `aios-cli proc kill <pid>`, \
                `aios-cli net ifconfig`. Do not output anything else in the block."
            }),
            serde_json::json!({
                "role": "user",
                "content": user_prompt
            }),
        ];

        // Locate the compiled `aios-cli` binary alongside the running `aios-daemon` binary once
        let exe_path = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("aios-daemon"));
        let mut cli_path = exe_path.parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
        #[cfg(target_os = "windows")]
        cli_path.push("aios-cli.exe");
        #[cfg(not(target_os = "windows"))]
        cli_path.push("aios-cli");
        let cli_path_str = cli_path.to_string_lossy().to_string();

        let client = reqwest::blocking::Client::new();

        for i in 0..5 {
            let payload = serde_json::json!({
                "model": model,
                "messages": messages,
                "stream": false
            });

            let res = client
                .post("http://127.0.0.1:11434/api/chat")
                .header("Content-Type", "application/json")
                .json(&payload)
                .send();

            match res {
                Ok(response) => {
                    if !response.status().is_success() {
                        return ExecutionResult {
                            success: false,
                            output: "".to_string(),
                            error: Some(format!("Ollama returned an HTTP {}", response.status())),
                        };
                    }

                    match response.json::<serde_json::Value>() {
                        Ok(json) => {
                            if let Some(content) = json["message"]["content"].as_str() {
                                if content.contains("[COMMAND]") {
                                    let start = content.find("[COMMAND]").unwrap() + 9;
                                    let command_str = if let Some(end) = content.find("[/COMMAND]") {
                                        &content[start..end]
                                    } else {
                                        &content[start..]
                                    };
                                    let command_str = command_str.trim();
                                    println!("Step {}: LLM decided to run: {}", i + 1, command_str);
                                    
                                    // Use replace() for ALL occurrences to support chained commands.
                                    let safe_cmd = command_str.replace("aios-cli", &format!("\"{}\"", cli_path_str));
                                    
                                    #[cfg(target_os = "windows")]
                                    let output_res = std::process::Command::new("cmd")
                                        .raw_arg(format!("/C {}", safe_cmd))
                                        .current_dir(&context.active_directory)
                                        .output();

                                    #[cfg(not(target_os = "windows"))]
                                    let output_res = std::process::Command::new("sh")
                                        .args(["-c", &safe_cmd])
                                        .current_dir(&context.active_directory)
                                        .output();

                                    match output_res {
                                        Ok(output) => {
                                            let stdout = String::from_utf8_lossy(&output.stdout);
                                            let stderr = String::from_utf8_lossy(&output.stderr);
                                            let final_out = if stderr.trim().is_empty() { stdout.to_string() } else { format!("{}\n{}", stdout, stderr) };
                                            
                                            // Append assistant response and tool output to history for next iteration
                                            messages.push(serde_json::json!({
                                                "role": "assistant",
                                                "content": content
                                            }));
                                            messages.push(serde_json::json!({
                                                "role": "user",
                                                "content": format!("COMMAND OUTPUT:\n{}", final_out)
                                            }));
                                            
                                            println!("Agentic loop continuing to next iteration...");
                                            continue;
                                        },
                                        Err(e) => {
                                            return ExecutionResult {
                                                success: false,
                                                output: "".to_string(),
                                                error: Some(format!("Failed to execute CLI Subprocess: {}", e)),
                                            };
                                        }
                                    }
                                }
                                
                                return ExecutionResult {
                                    success: true,
                                    output: content.to_string(),
                                    error: None,
                                };
                            } else {
                                return ExecutionResult {
                                    success: false,
                                    output: "".to_string(),
                                    error: Some("Malformed response from Ollama (no content)".to_string()),
                                };
                            }
                        }
                        Err(e) => return ExecutionResult {
                            success: false,
                            output: "".to_string(),
                            error: Some(format!("Failed to parse Ollama JSON: {}", e)),
                        },
                    }
                }
                Err(e) => return ExecutionResult {
                    success: false,
                    output: "".to_string(),
                    error: Some(format!("Request to Ollama failed: {}", e)),
                },
            }
        }

        ExecutionResult {
            success: false,
            output: "".to_string(),
            error: Some("Error: Agentic loop limit reached (5 iterations) without a final answer.".to_string()),
        }
    }
}
