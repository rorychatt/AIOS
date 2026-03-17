pub mod plugins;
pub mod llm_router;
pub mod process_manager;
pub mod network_manager;

use aios_core::init_core;
use aios_core::models::{Intent, SystemContext};
use aios_core::plugin::AiosNativeApp;
use std::collections::HashMap;

use plugins::FileSystemApp;
use llm_router::LlmRouterApp;
use process_manager::ProcessManagerApp;
use network_manager::NetworkManagerApp;

use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
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
                    
                    match serde_yaml::from_str::<Intent>(&request_str) {
                        Ok(intent) => {
                            println!("Received Intent via YAML: {:?}", intent);

                            // Dummy context
                            let context = SystemContext {
                                active_directory: "/workspace".to_string(),
                                user_id: "agent_01".to_string(),
                                permissions: vec![
                                    "fs.read".to_string(), 
                                    "api.openai".to_string(),
                                    "proc.manage".to_string(),
                                    "net.read".to_string()
                                ],
                            };

                            let mut final_result = aios_core::models::ExecutionResult {
                                success: false,
                                output: "".to_string(),
                                error: Some("Capability not handled".to_string()),
                            };

                            // Route Intents
                            let cap_str = intent.target_capability.as_deref().unwrap_or("");
                            
                            if cap_str == "List" || cap_str == "Read" {
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
                                
                                if llm_response.success && llm_response.output.starts_with("[ROUTE]:") {
                                    let mut new_intent = intent.clone();
                                    let predicted_cap = llm_response.output.replace("[ROUTE]:", "").trim().to_string();
                                    new_intent.target_capability = Some(predicted_cap.clone());
                                    
                                    println!("LLM suggested route: {}", predicted_cap);
                                    
                                    if predicted_cap == "List" || predicted_cap == "Read" {
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

