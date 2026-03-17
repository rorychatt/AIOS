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
                let mut buffer = [0; 512];
                if let Ok(size) = stream.read(&mut buffer) {
                    let request = String::from_utf8_lossy(&buffer[..size]);
                    println!("Received command from CLI: {}", request.trim());

                    // Dummy context
                    let context = SystemContext {
                        active_directory: "/workspace".to_string(),
                        user_id: "agent_01".to_string(),
                        permissions: vec!["fs.read".to_string()],
                    };

                    let intent = Intent {
                        raw_text: request.trim().to_string(),
                        target_capability: Some("List".to_string()), // hardcoded for demo
                        parameters: HashMap::new(),
                    };

                    let result = fs_plugin.execute(&intent, &context);
                    
                    let response = format!("Result: {}\n", result.output);
                    stream.write_all(response.as_bytes()).unwrap();
                }
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}

