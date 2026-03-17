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

    let command = args[1].clone();

    if command == "start" {
        println!("Booting AIOS...");
        start_aios();
        return;
    }

    if command == "stop" {
        stop_aios();
        return;
    }

    if args.len() < 3 {
        println!("Usage: aios-cli <subcommand> [args]");
        println!("       aios-cli start");
        println!("       aios-cli stop");
        println!("       aios-cli fs list <path>");
        println!("       aios-cli fs read <path>");
        println!("       aios-cli fs write <path> <content>");
        println!("       aios-cli fs create-folder <path>");
        println!("       aios-cli proc ps");
        println!("       aios-cli proc kill <pid>");
        println!("       aios-cli net ifconfig");
        return;
    }

    let module = args[1].clone();
    let action = args[2].clone();
    
    let mut intent = Intent {
        raw_text: args.join(" "),
        target_capability: None,
        parameters: HashMap::new(),
    };

    match module.as_str() {
        "fs" => {
            match action.as_str() {
                "list" => {
                    intent.target_capability = Some("List".to_string());
                    if args.len() > 3 {
                        intent.parameters.insert("path".to_string(), args[3].clone());
                    }
                }
                "read" => {
                    intent.target_capability = Some("Read".to_string());
                    if args.len() > 3 {
                        intent.parameters.insert("path".to_string(), args[3].clone());
                    }
                }
                "write" => {
                    intent.target_capability = Some("Write".to_string());
                    if args.len() > 3 {
                        intent.parameters.insert("path".to_string(), args[3].clone());
                    }
                    if args.len() > 4 {
                        intent.parameters.insert("content".to_string(), args[4].clone());
                    }
                }
                "create-folder" => {
                    intent.target_capability = Some("CreateFolder".to_string());
                    if args.len() > 3 {
                        intent.parameters.insert("path".to_string(), args[3].clone());
                    }
                }
                _ => {
                    println!("Unknown fs action: {}", action);
                    return;
                }
            }
        }
        "proc" => {
            match action.as_str() {
                "ps" => {
                    intent.target_capability = Some("Ps".to_string());
                }
                "kill" => {
                    intent.target_capability = Some("Kill".to_string());
                    if args.len() > 3 {
                        intent.parameters.insert("pid".to_string(), args[3].clone());
                    }
                }
                _ => {
                    println!("Unknown proc action: {}", action);
                    return;
                }
            }
        }
        "net" => {
             match action.as_str() {
                "ifconfig" => {
                    intent.target_capability = Some("IfConfig".to_string());
                }
                 _ => {
                    println!("Unknown net action: {}", action);
                    return;
                }
             }
        }
        _ => {
            // Direct capability mode fallback (Legacy)
            intent.target_capability = Some(module);
            intent.raw_text = action;
            for arg in args.iter().skip(3) {
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                if parts.len() == 2 {
                    intent.parameters.insert(parts[0].to_string(), parts[1].to_string());
                }
            }
        }
    }

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

fn start_aios() {
    use std::process::Command;
    use std::thread;
    use std::time::Duration;

    // 1. Start the daemon in the background
    println!("Starting aios-daemon in the background...");
    let mut daemon_child = match Command::new("cargo")
        .arg("run")
        .arg("--bin")
        .arg("aios-daemon")
        .current_dir(env::current_dir().unwrap_or_else(|_| ".".into()))
        .spawn() 
    {
        Ok(child) => {
            println!("Daemon started successfully.");
            child
        },
        Err(e) => {
            println!("Failed to start daemon: {}", e);
            return;
        }
    };

    // Give daemon a second to bind to port 9090
    thread::sleep(Duration::from_secs(2));

    // 2. Start the aios-dashboard
    let dashboard_dir = env::current_dir().unwrap_or_else(|_| ".".into()).join("aios-dashboard");
    println!("Starting aios-dashboard...");
    let mut dashboard_child = match Command::new("dotnet")
        .arg("run")
        .current_dir(&dashboard_dir)
        .spawn()
    {
        Ok(child) => {
            println!("Dashboard web server started (port 5010).");
            child
        },
        Err(e) => {
            println!("Failed to start dashboard: {}", e);
            let _ = daemon_child.kill();
            return;
        }
    };

    // Give the Web Server a second to initialize
    thread::sleep(Duration::from_secs(2));

    // 3. Launch the browser
    println!("Opening Ivy Framework Dashboard...");
    if let Err(e) = open::that("http://localhost:5010") {
        println!("Failed to open browser: {}", e);
        println!("Please navigate manually to http://localhost:5010");
    }

    println!("===========================================================");
    println!("AIOS is running! Press Ctrl+C in this terminal to stop it.");
    println!("===========================================================");

    // 4. Block so the processes tie to this terminal
    let _ = daemon_child.wait();
    let _ = dashboard_child.wait();
    
    println!("AIOS processes stopped.");
}

fn stop_aios() {
    use std::process::Command;
    println!("Stopping AIOS background processes...");
    
    // Windows taskkill
    let _ = Command::new("taskkill").args(&["/F", "/IM", "aios-daemon.exe"]).status();
    let _ = Command::new("taskkill").args(&["/F", "/IM", "AiosDashboard.exe"]).status();
    
    println!("Dangling processes cleared.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_has_start_and_stop_functions() {
        // Just verify they compile and exist, actual execution launches heavy processes
        let start_fn: fn() = start_aios;
        let stop_fn: fn() = stop_aios;
        
        assert!(start_fn as usize > 0);
        assert!(stop_fn as usize > 0);
    }

    #[test]
    fn test_intent_struct_serialization() {
        let intent = Intent {
            raw_text: "Read file".to_string(),
            target_capability: Some("Read".to_string()),
            parameters: std::collections::HashMap::from([
                ("path".to_string(), "foo.txt".to_string())
            ]),
        };
        
        let yaml = serde_yaml::to_string(&intent).unwrap();
        assert!(yaml.contains("Read file"));
        assert!(yaml.contains("foo.txt"));
        assert!(yaml.contains("target_capability"));
    }
}
