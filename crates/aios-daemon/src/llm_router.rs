use aios_core::models::{ExecutionResult, Intent, SystemContext};
use aios_core::plugin::AiosNativeApp;
use serde_yaml;

pub struct LlmRouterApp;

impl AiosNativeApp for LlmRouterApp {
    fn id(&self) -> &str {
        "core.llm.router"
    }

    fn describe_capabilities(&self) -> Vec<String> {
        vec![
            "Route intent to OpenAI for reasoning [Route(intent_text)]".to_string(),
        ]
    }

    fn execute(&self, intent: &Intent, _context: &SystemContext) -> ExecutionResult {
        // Here we handle the bridging to OpenAI Chat completions API.
        
        let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_default();
        if api_key.is_empty() {
             return ExecutionResult {
                success: false,
                output: "".to_string(),
                error: Some("OPENAI_API_KEY environment variable is not set.".to_string()),
            };
        }

        let user_prompt = intent.parameters.get("intent_text").unwrap_or(&intent.raw_text);

        let payload = serde_yaml::to_string(&serde_yaml::Mapping::from_iter(vec![
            (
                serde_yaml::Value::String("model".to_string()),
                serde_yaml::Value::String("gpt-4-turbo".to_string())
            ),
            (
                serde_yaml::Value::String("messages".to_string()),
                serde_yaml::Value::Sequence(vec![
                    serde_yaml::Value::Mapping({
                        let mut m = serde_yaml::Mapping::new();
                        m.insert(serde_yaml::Value::String("role".to_string()), serde_yaml::Value::String("system".to_string()));
                        m.insert(serde_yaml::Value::String("content".to_string()), serde_yaml::Value::String("You are AIOS, the conversational AI-first Operating System kernel. The user is talking to you via a terminal. You can answer their questions naturally. If they ask to perform an action (like listing files, viewing network, or checking processes), you MUST reply EXACTLY with `[ROUTE]: <Capability>` (e.g. `[ROUTE]: Ps`, `[ROUTE]: List`, `[ROUTE]: IfConfig`). DO NOT wrap it in backticks, just output the string. If no action is needed, just converse with the user.".to_string()));
                        m
                    }),
                    serde_yaml::Value::Mapping({
                        let mut m = serde_yaml::Mapping::new();
                        m.insert(serde_yaml::Value::String("role".to_string()), serde_yaml::Value::String("user".to_string()));
                        m.insert(serde_yaml::Value::String("content".to_string()), serde_yaml::Value::String(user_prompt.to_string()));
                        m
                    })
                ])
            )
        ])).unwrap();

        // Blocking HTTP call to OpenAI
        let client = reqwest::blocking::Client::new();
        let res = client.post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json") // API expects JSON
            // We use a quick hack to convert the YAML Payload structure to JSON for the API request here
            .body(serde_json::to_string(&serde_yaml::from_str::<serde_json::Value>(&payload).unwrap()).unwrap())
            .send();

        match res {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<serde_json::Value>() {
                        Ok(json) => {
                            if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                                ExecutionResult {
                                    success: true,
                                    output: content.to_string(), // Return the conversational or route string directly
                                    error: None,
                                }
                            } else {
                                ExecutionResult {
                                    success: false,
                                    output: "".to_string(),
                                    error: Some("Malformed response from OpenAI".to_string()),
                                }
                            }
                        },
                        Err(e) => ExecutionResult {
                            success: false,
                            output: "".to_string(),
                            error: Some(format!("Failed to parse OpenAI JSON: {}", e)),
                        }
                    }
                } else {
                     ExecutionResult {
                        success: false,
                        output: "".to_string(),
                        error: Some(format!("OpenAI returned an HTTP {}", response.status())),
                    }
                }
            },
            Err(e) => ExecutionResult {
                success: false,
                output: "".to_string(),
                error: Some(format!("Request to OpenAI failed: {}", e)),
            }
        }
    }
}
