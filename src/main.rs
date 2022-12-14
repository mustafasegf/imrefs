use std::env::args;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::process::{exit, Command};
// use std::process::Stdio;

fn main() {
    let args = args().collect::<Vec<_>>();

    let cmd = match args.len() {
        1 => {
            println!("Usage: executable [command]");
            exit(1);
        }
        2 => args[1].clone(),
        _ => args[1].clone(),
    };

    match cmd.as_str() {
        "init" => {
            if args.len() != 3 {
                println!("Usage: executable init [name]");
                exit(1);
            }
            let name = args[2].clone();

            let Ok(child) = Command::new("/proc/self/exe")
                .arg("child")
                .arg(&name)
                // .stdout(Stdio::null())
                .spawn() else {
                    println!("Failed to spawn child");
                    exit(1);
                };

            let file_name = format!("/tmp/imrefs-{}.tmp", name);
            if let Err(e) = File::create(&file_name) {
                println!("Error can't create file: {}", e);
                exit(1);
            };

            println!(
                "Filesystem {} successfully created at {} with PID {}",
                &name,
                &file_name,
                child.id()
            );
        }
        "send" => {
            if args.len() != 4 {
                println!("Usage: executable send [name] [message]");
                exit(1);
            }
            let name = args[2].clone();
            let message = format!("msg:{}", args[3].clone());
            let socket_path = format!("/tmp/imrefs-{}.sock", name);
            let mut stream = match UnixStream::connect(&socket_path) {
                Ok(s) => s,
                Err(_) => {
                    println!("Filesystem {} not found", &name);
                    exit(1);
                }
            };

            if let Err(e) = stream.write_all(message.as_bytes()) {
                println!("Error can't write to socket: {}", e);
                exit(1);
            };
        }
        "stop" => {
            if args.len() != 3 {
                println!("Usage: executable stop [name]");
                exit(1);
            }

            let name = args[2].clone();
            let socket_path = format!("/tmp/imrefs-{}.sock", name);
            let mut stream = match UnixStream::connect(&socket_path) {
                Ok(s) => s,
                Err(_) => {
                    println!("Filesystem {} not found", &name);
                    exit(1);
                }
            };

            if let Err(e) = stream.write_all("cmd:stop".as_bytes()) {
                println!("Error can't write to socket: {}", e);
                exit(1);
            };
        }

        "child" => {
            if args.len() != 3 {
                println!("Usage: executable child [name]");
                exit(1);
            }
            let name = args[2].clone();
            let socket_path = format!("/tmp/imrefs-{}.sock", name);
            let file_name = format!("/tmp/imrefs-{}.tmp", name);

            if std::fs::metadata(&socket_path).is_ok() {
                if let Err(e) = fs::remove_file(&socket_path) {
                    println!("Error can't remove socket: {}", e);
                    exit(1);
                };
            }

            let listener = match UnixListener::bind(&socket_path) {
                Ok(l) => l,
                Err(_) => {
                    println!("Error can't bind socket");
                    exit(1);
                }
            };

            for stream in listener.incoming() {
                let mut stream = match stream {
                    Ok(stream) => stream,
                    Err(e) => {
                        println!("Error: {}", e);
                        continue;
                    }
                };
                let mut response = String::new();
                if let Err(e) = stream.read_to_string(&mut response) {
                    println!("Error can't read from socket: {}", e);
                    continue;
                };

                if response == "cmd:stop" {
                    if let Err(e) = fs::remove_file(&file_name) {
                        println!("Error can't remove file: {}", e);
                        exit(1);
                    };
                    if let Err(e) = fs::remove_file(&socket_path) {
                        println!("Error can't remove file: {}", e);
                        exit(1);
                    };

                    println!("Filesystem {} successfully removed", &name);
                    exit(0);
                }

                response = response.replace("msg:", "");

                if Path::new(&file_name).exists() {
                    if let Err(e) = fs::remove_file(&file_name) {
                        println!("Error can't remove file: {}", e);
                        continue;
                    };
                }

                let mut file = match OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .append(true)
                    .open(&file_name)
                {
                    Ok(f) => f,
                    Err(e) => {
                        println!("Error can't open file: {}", e);
                        exit(1);
                    }
                };

                if let Err(e) = file.write_all(response.as_bytes()) {
                    println!("Error can't write to file: {}", e);
                    continue;
                };

                println!("Data successfully written to file: {}", &file_name);
                exit(0);
            }
        }
        _ => {
            println!("Usage: executable [command] where command is one of init, send, stop");
            exit(1);
        }
    };
}
