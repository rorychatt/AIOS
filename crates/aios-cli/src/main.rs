use std::io::{Read, Write};
use std::net::TcpStream;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: aios-cli <intent statement>");
        println!("Example: aios-cli \"List files in directory\"");
        return;
    }

    let intent = args[1..].join(" ");
    println!("Calling AIOS Daemon with Intent: '{}'", intent);

    match TcpStream::connect("127.0.0.1:9090") {
        Ok(mut stream) => {
            stream.write_all(intent.as_bytes()).unwrap();

            let mut buffer = String::new();
            if let Ok(_) = stream.read_to_string(&mut buffer) {
                println!("{}", buffer);
            }
        }
        Err(e) => {
            println!("Failed to connect to daemon: {}", e);
            println!("Make sure aios-daemon is running.");
        }
    }
}
