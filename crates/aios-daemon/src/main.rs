pub mod llm_router;
pub mod network_manager;
pub mod plugins;
pub mod process_manager;

use aios_core::init_core;
use aios_core::models::{Intent, SystemContext};
use aios_core::plugin::AiosNativeApp;

use llm_router::LlmRouterApp;
use network_manager::NetworkManagerApp;
use plugins::FileSystemApp;
use process_manager::ProcessManagerApp;

use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    println!("Starting AIOS Daemon...");
    init_core();

    let fs_plugin = FileSystemApp;
    let proc_plugin = ProcessManagerApp;
    let net_plugin = NetworkManagerApp;

    println!("Loaded Plugin: {}", fs_plugin.id());
    println!("Loaded Plugin: {}", proc_plugin.id());
    println!("Loaded Plugin: {}", net_plugin.id());

    let listener = TcpListener::bind("127.0.0.1:9090").expect("Failed to bind to port 9090");
    println!("Daemon listening on 127.0.0.1:9090...");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 4096];
                if let Ok(size) = stream.read(&mut buffer) {
                    let request_str = String::from_utf8_lossy(&buffer[..size]);
                    
                    let clean_request = request_str.split("---").next().unwrap_or(&request_str).trim();

                    match serde_yaml::from_str::<Intent>(clean_request) {
                        Ok(intent) => {
                            println!("Received Intent via YAML: {:?}", intent);

                            // Sandboxed context
                            let mut current_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
                            current_dir.push(".aios");
                            if !current_dir.exists() {
                                std::fs::create_dir_all(&current_dir).unwrap_or_else(|_| println!("Warning: Failed to create ~/.aios"));
                            }
                            
                            let context = SystemContext {
                                active_directory: current_dir.to_string_lossy().to_string(),
                                user_id: "agent_01".to_string(),
                                permissions: vec![
                                    "fs.read".to_string(),
                                    "fs.write".to_string(),
                                    "api.openai".to_string(),
                                    "proc.manage".to_string(),
                                    "net.read".to_string(),
                                ],
                            };

                            let final_result;

                            // Route Intents
                            let cap_str = intent.target_capability.as_deref().unwrap_or("");

                            if cap_str == "List" || cap_str == "Read" || cap_str == "Write" {
                                final_result = fs_plugin.execute(&intent, &context);
                            } else if cap_str == "Ps" || cap_str == "Kill" {
                                final_result = proc_plugin.execute(&intent, &context);
                            } else if cap_str == "IfConfig" {
                                final_result = net_plugin.execute(&intent, &context);
                            } else {
                                // Fallback: Route through LLM!
                                println!("No exact match for capability, routing to LLM...");
                                let router = LlmRouterApp;
                                let llm_response = router.execute(&intent, &context);

                                if llm_response.success
                                    && llm_response.output.starts_with("[ROUTE]:")
                                {
                                    let mut new_intent = intent.clone();
                                    
                                    let output_str = llm_response.output.replace("[ROUTE]:", "").trim().to_string();
                                    let parts: Vec<&str> = output_str.splitn(2, '|').collect();
                                    
                                    let predicted_cap = parts[0].trim().to_string();
                                    new_intent.target_capability = Some(predicted_cap.clone());
                                    
                                    if parts.len() > 1 {
                                        if let Ok(parsed_json) = serde_json::from_str::<serde_json::Value>(parts[1].trim()) {
                                            if let Some(obj) = parsed_json.as_object() {
                                                for (k, v) in obj {
                                                    if let Some(s) = v.as_str() {
                                                        new_intent.parameters.insert(k.clone(), s.to_string());
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    println!("LLM suggested route: {} with parameters: {:?}", predicted_cap, new_intent.parameters);

                                    if predicted_cap == "List" || predicted_cap == "Read" || predicted_cap == "Write" {
                                        final_result = fs_plugin.execute(&new_intent, &context);
                                    } else if predicted_cap == "Ps" || predicted_cap == "Kill" {
                                        final_result = proc_plugin.execute(&new_intent, &context);
                                    } else if predicted_cap == "IfConfig" {
                                        final_result = net_plugin.execute(&new_intent, &context);
                                    } else {
                                        final_result = llm_response; // Give conversational output back
                                    }
                                } else {
                                    final_result = llm_response; // pure conversational response
                                }
                            }

                            let mut response_yaml = serde_yaml::to_string(&final_result).unwrap();
                            response_yaml.push_str("\n---\n"); // YAML document separator for framing
                            stream.write_all(response_yaml.as_bytes()).unwrap();
                        }
                        Err(e) => {
                            println!("Failed to parse intent YAML: {}", e);
                            let error_result = aios_core::models::ExecutionResult {
                                success: false,
                                output: "".to_string(),
                                error: Some(format!("Invalid YAML format: {}", e)),
                            };
                            let mut response_yaml = serde_yaml::to_string(&error_result).unwrap();
                            response_yaml.push_str("\n---\n");
                            stream.write_all(response_yaml.as_bytes()).unwrap();
                        }
                    }
                }
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}
