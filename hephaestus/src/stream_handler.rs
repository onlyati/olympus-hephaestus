use std::mem::size_of;
use std::io::BufReader;
use std::os::unix::net::UnixStream;
use std::io::Read;
use std::io::Write;

pub fn handle_client(mut stream: UnixStream) {
    let buffer = BufReader::new(&stream);

    let mut length_u8: Vec<u8> = Vec::with_capacity(5 * size_of::<usize>());
    let mut msg_u8: Vec<u8> = Vec::new();
    let mut length: usize = 0;
    let mut index = 0;
    let mut read_msg: bool = false;

    for byte in buffer.bytes() {
        match byte {
            Ok(b) => {
                if b == b' ' && !read_msg {
                    let msg_len_t = String::from_utf8(length_u8.clone()).unwrap();
                    length = match msg_len_t.parse::<usize>() {
                        Ok(v) => v,
                        Err(_) => {
                            let _ = stream.write_all(b"First word must be a number which is the lenght of message");
                            return;
                        }
                    };
                    msg_u8 = Vec::with_capacity(length);
                    read_msg = true;
                    continue;
                }

                if b == b'\n' {
                    break;
                }

                if read_msg {
                    msg_u8.push(b);
                    index += 1;
                    if index == length {
                        break;
                    }
                    continue;
                }
                else {
                    length_u8.push(b);
                    continue;
                }
            },
            Err(e) => println!("Unexpected error: {:?}", e),
        }
    }

    if !read_msg {
        let _ = stream.write_all(b"First word must be a number which is the lenght of message");
        return;
    }

    let command = String::from_utf8(msg_u8).unwrap();
    println!("Incoming request was: [{}]", command);

    let _ = stream.write_all(b"OK");
}