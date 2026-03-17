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

    fn execute(&self, intent: &Intent, _context: &SystemContext) -> ExecutionResult {
        let model = std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3".to_string());
        let user_prompt = intent
            .parameters
            .get("intent_text")
            .unwrap_or(&intent.raw_text);

        // We use serde_json for the HTTP payload natively now that we don't need to force YAML structures through the API layer
        let payload = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are AIOS, the conversational AI-first Operating System kernel. The user is talking to you via a terminal. You can answer their questions naturally. If they ask to perform an action (like reading/writing files, viewing network, or checking processes), you MUST reply EXACTLY with `[ROUTE]: <Capability> | <JSON_Parameters>`. Example: `[ROUTE]: Write | {\"path\": \"file.txt\", \"content\": \"hello\"}` or `[ROUTE]: Ps | {}`. Available capabilities: List, Read, Write, Ps, Kill, IfConfig. Do not output anything else if you are routing an action."
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
                                ExecutionResult {
                                    success: true,
                                    output: content.to_string(), // Return the conversational or route string directly
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
