use aios_core::models::{ExecutionResult, Intent, SystemContext};
use aios_core::plugin::AiosNativeApp;

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

        let payload = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are AIOS, the conversational AI-first Operating System kernel. The user is talking to you via a terminal. You can answer their questions naturally. If you need to perform actions on the OS itself to answer their question, output the exact CLI command to run inside a `[COMMAND]` block, like: `[COMMAND] aios-cli fs list . [/COMMAND]`. Available commands: \
                    `aios-cli fs list <path>`, \
                    `aios-cli fs read <file>`, \
                    `aios-cli fs write <path> <content>` (use this BOTH for creating new files and modifying existing ones), \
                    `aios-cli fs create-folder <path>`, \
                    `aios-cli proc ps`, \
                    `aios-cli proc kill <pid>`, \
                    `aios-cli net ifconfig`. Do not output anything else in the block."
                },
                {
                    "role": "user",
                    "content": user_prompt
                }
            ],
            "stream": false
        });

        // Blocking HTTP call to local Ollama
        let client = reqwest::blocking::Client::new();
        let res = client
            .post("http://127.0.0.1:11434/api/chat")
            .header("Content-Type", "application/json")
            .json(&payload)
            .send();

        match res {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>() {
                        Ok(json) => {
                            if let Some(content) = json["message"]["content"].as_str() {
                                // If the LLM successfully emitted a command block, we intercept it here to run the sub-process!
                                if content.contains("[COMMAND]") {
                                    let start = content.find("[COMMAND]").unwrap() + 9;
                                    let command_str = if let Some(end) = content.find("[/COMMAND]") {
                                        &content[start..end]
                                    } else {
                                        &content[start..]
                                    };
                                    let command_str = command_str.trim();
                                    println!("LLM decided to run: {}", command_str);
                                    
                                    // Replace `aios-cli` with `cargo run --bin aios-cli --` so we use the dev environment
                                    // and let the OS shell perfectly parse quotes and escape characters like \ (spaces)
                                    let safe_cmd = command_str.replacen("aios-cli", "cargo run --quiet --bin aios-cli --", 1);
                                    
                                    #[cfg(target_os = "windows")]
                                    let output_res = std::process::Command::new("cmd")
                                        .args(["/C", &safe_cmd])
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
                                            
                                            return ExecutionResult {
                                                success: true,
                                                output: final_out,
                                                error: None,
                                            };
                                        },
                                        Err(e) => {
                                            return ExecutionResult {
                                                success: false,
                                                output: "".to_string(),
                                                error: Some(format!("Failed to execute CLI Subprocess via shell: {}", e)),
                                            };
                                        }
                                    }
                                }
                                
                                ExecutionResult {
                                    success: true,
                                    output: content.to_string(),
                                    error: None,
                                }
                            } else {
                                ExecutionResult {
                                    success: false,
                                    output: "".to_string(),
                                    error: Some("Malformed response from Ollama".to_string()),
                                }
                            }
                        }
                        Err(e) => ExecutionResult {
                            success: false,
                            output: "".to_string(),
                            error: Some(format!("Failed to parse Ollama JSON: {}", e)),
                        },
                    }
                } else {
                    ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some(format!("Ollama returned an HTTP {}", response.status())),
                    }
                }
            }
            Err(e) => ExecutionResult {
                success: false,
                output: "".to_string(),
                error: Some(format!("Request to OpenAI failed: {}", e)),
            },
        }
    }
}
