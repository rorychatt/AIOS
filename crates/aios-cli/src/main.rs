use std::io::{Read, Write};
use std::net::TcpStream;
use std::env;
use std::collections::HashMap;
use aios_core::models::{Intent, ExecutionResult};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: aios-cli <intent statement>");
        println!("Example: aios-cli \"List files in directory\"");
        return;
    }

    let raw_text = args[1..].join(" ");
    let intent = Intent {
        raw_text: raw_text.clone(),
        target_capability: Some("List".to_string()), // Hardcode mapping for CLI testing right now
        parameters: HashMap::new(),
    };

    println!("Calling AIOS Daemon with Intent: '{}'", raw_text);

    match TcpStream::connect("127.0.0.1:9090") {
        Ok(mut stream) => {
            let mut request_yaml = serde_yaml::to_string(&intent).unwrap();
            request_yaml.push_str("\n---\n");
            stream.write_all(request_yaml.as_bytes()).unwrap();

            let mut buffer = String::new();
            if let Ok(_) = stream.read_to_string(&mut buffer) {
                // Strip the exact framing if necessary, though yaml parser usually ignores trailing breaks
                match serde_yaml::from_str::<ExecutionResult>(&buffer) {
                    Ok(result) => {
                        if result.success {
                            println!("Success:\n{}", result.output);
                        } else {
                            println!("Error:\n{}", result.error.unwrap_or_else(|| "Unknown Error".to_string()));
                        }
                    }
                    Err(e) => {
                        println!("YAML Parse Error: {}", e);
                        // Fallback to raw buffer
                        println!("Raw Response:\n{}", buffer);
                    }
                }
            }
        }
        Err(e) => {
            println!("Failed to connect to daemon: {}", e);
            println!("Make sure aios-daemon is running.");
        }
    }
}
