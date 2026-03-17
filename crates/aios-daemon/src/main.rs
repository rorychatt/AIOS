pub mod plugins;

use aios_core::init_core;
use aios_core::models::{Intent, SystemContext};
use aios_core::plugin::AiosNativeApp;
use std::collections::HashMap;
use plugins::FileSystemApp;

use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    println!("Starting AIOS Daemon...");
    init_core();

    let fs_plugin = FileSystemApp;
    println!("Loaded Plugin: {}", fs_plugin.id());

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
                                permissions: vec!["fs.read".to_string()],
                            };

                            let result = fs_plugin.execute(&intent, &context);
                            
                            let mut response_yaml = serde_yaml::to_string(&result).unwrap();
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

