use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::process::exit;
use std::env;
use std::path::Path;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Missing input parameters!");
        exit(1);
    }

    args.remove(0);

    if args[0] == "--version" {
        println!("v.0.1.1");
        exit(0);
    }

    // Execute script dynmically
    if args[0] == "-x" {
        if args.len() < 2 {
            println!("No script is supplied!\n");
            exit(1);
        }
        let file = args[args.len() - 1].clone();
        let content = match std::fs::read_to_string(&file) {
            Ok(c) => c,
            Err(e) => {
                println!("{:?}\n", e);
                exit(1);
            },
        };

        let quals: Vec<&str> = file.split("/").collect();
        let file = String::from(quals[quals.len() - 1]);

        match std::fs::write(format!("/etc/olympus/hephaestus/plans/console/{}.conf", file), content) {
            Ok(_) => {
                let cmd = std::process::Command::new("hephaestus-cli")
                    .arg("exec")
                    .arg("console")
                    .arg(&file[..])
                    .output();

                match cmd {
                    Ok(o) => {
                        println!("{}", String::from_utf8(o.stdout).unwrap());
                    },
                    Err(e) => println!("{:?}\n", e),
                }
            },
            Err(e) => {
                println!("{:?}\n", e);
                exit(1);
            },
        }

        let _ = std::fs::remove_file(format!("/etc/olympus/hephaestus/plans/console/{}.conf", file));

        return;
    }

    // Forward requst to server
    let socket = if args[0] == "--dev" {
        args.remove(0);
        Path::new("/tmp/hephaestus-dev.sock")
    } else {
        Path::new("/tmp/hephaestus.sock")
    };

    let mut message = String::new();
    for arg in args {
        message += &arg[..];
        message += " ";
    }

    let count = message.len();

    let message = format!("{} {}", count, message);

    let mut stream = match UnixStream::connect(socket) {
        Ok(v) => v,
        Err(e) => {
            println!("Error during connect to socket: {e:?}");
            exit(1);
        }
    };

    if let Err(e) = stream.write_all(&message.as_bytes()) {
        println!("Error during sending data: {e:?}");
    }

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    println!("{response}");
}
