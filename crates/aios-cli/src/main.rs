use aios_core::models::{ExecutionResult, Intent};
use std::collections::HashMap;
use std::env;
use std::io::{self, Read, Write};
use std::net::TcpStream;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Starting AIOS Conversational Mode...");
        println!("Type 'exit' to quit.");
        run_repl();
        return;
    }

    // Direct CLI execution mode
    let target_cap = args[1].clone();
    let raw_text = args[2].clone();

    let mut params = HashMap::new();
    for arg in args.iter().skip(3) {
        let parts: Vec<&str> = arg.splitn(2, '=').collect();
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
    send_intent(intent);
}

fn run_repl() {
    let stdin = io::stdin();
    loop {
        print!("\nAIOS> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if stdin.read_line(&mut input).is_ok() {
            let input = input.trim();
            if input.is_empty() {
                continue;
            }
            if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
                println!("Goodbye!");
                break;
            }

            let intent = Intent {
                raw_text: input.to_string(),
                target_capability: None, // Pass to LLM router to figure out
                parameters: HashMap::new(),
            };

            send_intent(intent);
        }
    }
}

fn send_intent(intent: Intent) {
    match TcpStream::connect("127.0.0.1:9090") {
        Ok(mut stream) => {
            let mut request_yaml = serde_yaml::to_string(&intent).unwrap();
            request_yaml.push_str("\n---\n");
            stream.write_all(request_yaml.as_bytes()).unwrap();

            let mut buffer = String::new();
            if stream.read_to_string(&mut buffer).is_ok() {
                // Strip the YAML document separator if it exists so serde_yaml can parse it as a single document
                let clean_buffer = buffer.split("---").next().unwrap_or(&buffer).trim();
                match serde_yaml::from_str::<ExecutionResult>(clean_buffer) {
                    Ok(result) => {
                        if result.success {
                            println!("{}", result.output);
                        } else {
                            println!(
                                "Error: {}",
                                result.error.unwrap_or_else(|| "Unknown Error".to_string())
                            );
                        }
                    }
                    Err(e) => {
                        println!("YAML Parse Error: {}", e);
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
