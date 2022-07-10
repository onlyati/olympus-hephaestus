use std::mem::size_of;
use std::io::BufReader;
use std::os::unix::net::UnixStream;
use std::io::Read;
use std::io::Write;
use std::time::Duration;
use std::sync::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

use crate::commands;

/// Handle client requests
/// 
/// This function is intended to handle client requests from UNIX socket.
/// This function will reply on the stream where the data was incoming.
pub fn handle_client(mut stream: UnixStream, history: Arc<Mutex<HashMap<u64, Vec<String>>>>) {
    let buffer = BufReader::new(&stream);

    let mut length_u8: Vec<u8> = Vec::with_capacity(5 * size_of::<usize>());   // Store bytes while readin, itis the message length
    let mut length: usize = 0;                                                 // This will be the parsed lenght from length_u8

    let mut msg_u8: Vec<u8> = Vec::new();                                      // Store message bytes

    let mut index = 0;                                                         // Index and read_msg are some variable for parsing incoming message
    let mut read_msg: bool = false;

    /*-------------------------------------------------------------------------------------------*/
    /* Read message from the buffer and parse it accordingly                                     */
    /*-------------------------------------------------------------------------------------------*/
    for byte in buffer.bytes() {
        match byte {
            Ok(b) => {
                /* It was the first space, first word must be a number which is the length of the subsequent message */
                if b == b' ' && !read_msg {
                    let msg_len_t = String::from_utf8(length_u8.clone()).unwrap();
                    length = match msg_len_t.parse::<usize>() {
                        Ok(v) => v,
                        Err(_) => {
                            let _ = stream.write_all(b"First word must be a number which is the lenght of message\n");
                            return;
                        }
                    };
                    msg_u8 = Vec::with_capacity(length);
                    read_msg = true;
                    continue;
                }

                // Set timeout to avoid infinite waiting on the stream
                stream.set_read_timeout(Some(Duration::new(0, 250))).unwrap();

                /* Read from buffer */
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
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                let _ = stream.write_all(b"ERROR: Request is not complete within time\n");
                return;
            },
            Err(e) => {
                println!("Unexpected error: {:?}", e);
                let _ = stream.write_all(b"ERROR: Internal server error during stream reading\n");
                return;
            },
        }
    }

    if !read_msg {
        /* This happen when the first world was not a number and new line was incoming */
        let _ = stream.write_all(b"First word must be a number which is the lenght of message\n");
        return;
    }

    /*-------------------------------------------------------------------------------------------*/
    /* Readin from buffer was okay, now parse it then call the command coordinator and return    */
    /* with the answer of the command                                                            */
    /*-------------------------------------------------------------------------------------------*/
    let command = String::from_utf8(msg_u8).unwrap();

    let mut verb: String = String::from("");
    let mut options: Vec<String> = Vec::with_capacity(5 * size_of::<String>());

    let mut index = 0;
    for word in command.split_whitespace() {
        if index == 0 {
            verb = String::from(word);
        }
        else {
            options.push(String::from(word));
        }
        index += 1;
    }

    match command_coordinator(verb, options, history) {
        Ok(s) => {
            let _ = stream.write_all(s.as_bytes());
        },
        Err(e) => {
            let error_msg = format!("ERROR: {}", e);
            let _ = stream.write_all(error_msg.as_bytes());
        }
    }
}

/// Command coordinator
/// 
/// This function is called from `handle_client` function. This function will call the proper function for specified commands.
/// 
/// # Return values
/// If everything was cool, it return with `Ok(String)` where the string is the reply from the function.
/// If the called function return with error, it will return with the same `Err(String)`.
/// If command verb would not exist, it returns with `Error(String)`.
fn command_coordinator(verb: String, options: Vec<String>, history: Arc<Mutex<HashMap<u64, Vec<String>>>>) -> Result<String, String> {
    let list_verb = String::from("list");
    let exec_verb = String::from("exec");
    let stat_verb = String::from("status");
    let help_verb = String::from("help");
    let flow_verb = String::from("plans");
    let dump_verb = String::from("dump");

    if verb == list_verb {
        return commands::list(options);
    }

    if verb == exec_verb {
        return commands::exec(options, history);
    }

    if verb == stat_verb {
        return commands::status(options, history);
    }

    if verb == help_verb {
        return commands::help(options);
    }

    if verb == flow_verb {
        return commands::list_ids(options, history);
    }

    if verb == dump_verb {
        return commands::dump(options, history);
    }

    return Err(String::from("Invalid command verb"));
}