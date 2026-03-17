use std::io::{Read, Write};
use std::net::TcpStream;
use std::env;
use std::collections::HashMap;
use aios_core::models::{Intent, ExecutionResult};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: aios-cli <Capability> <Intent Statement> [paramKey=paramValue...]");
        println!("Example: aios-cli List \"List files in directory\" path=.");
        println!("Example: aios-cli Kill \"Kill process\" pid=1234");
        return;
    }

    let target_cap = args[1].clone();
    let raw_text = args[2].clone();
    
    let mut params = HashMap::new();
    for i in 3..args.len() {
        let parts: Vec<&str> = args[i].splitn(2, '=').collect();
        if parts.len() == 2 {
            params.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    let intent = Intent {
        raw_text: raw_text.clone(),
        target_capability: Some(target_cap), 
        parameters: params,
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
