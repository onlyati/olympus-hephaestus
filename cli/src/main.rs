use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::process::exit;
use std::env;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Missing input parameters!");
        exit(1);
    }

    args.remove(0);

    let mut message = String::new();
    for arg in args {
        message += &arg[..];
        message += " ";
    }

    let count = message.len();

    let message = format!("{} {}", count, message);

    let mut stream = match UnixStream::connect("/tmp/hephaestus.sock") {
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
