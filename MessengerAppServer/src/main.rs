#![allow(non_snake_case)]
#![allow(unused_imports)]

// Some of this code came from
// https://riptutorial.com/rust/example/4404/a-simple-tcp-client-and-server-application--echo

use std::thread;
use std::thread::sleep;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Write, ErrorKind, Error};
use std::io;
use MessengerAppServer::messages;
use MessengerAppServer::CONN;

// The IP_ADDRESS needs to be constantly updated depending on the machine this program is running on.
const IP_ADDRESS:&str = "192.168.5.248";
const PORT_NUMBER:&str = "8888";

fn main() {
    // This function starts the listener and then spawns a thread for each stream.

    messages::init_messages();

    let listener = TcpListener::bind(format!("{}:{}", IP_ADDRESS, PORT_NUMBER)).unwrap();

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move|| {
                    handle_client(stream)
                    // connection succeeded
                });
            }
            Err(e) => {
                println!("Error: {}", e);
                // connection failed
            }
        }
    }
    // close the socket server
    drop(listener);
}

fn handle_client(mut stream: TcpStream) {
    // READ STREAM
    // GET_MESSAGE_RESPONSE
    // WRITE STREAM

    let mut data = [0 as u8; 512]; // using 512 byte buffer

    match stream.read(&mut data) {
        Ok(_) => {
            let mut message = String::from_utf8_lossy(&data).into_owned();

            println!("{}", message);

            // This trim command gets rid of the null characters from the buffer.
            message = String::from(message.trim_matches(char::from(0)));

            let response = get_message_response(
                message).expect("There is a problem with get_message_response().");
            println!("{}", &response);
            stream.write(response.as_bytes()).unwrap();
        },
        Err(_) => {
            println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
            stream.shutdown(Shutdown::Both).unwrap();
        }
    }
}

// NOT TESTED
fn get_message_response(message: String) -> io::Result<String> {
    // This function simply identifies what kind of command this is
    // and then calls the corresponding function.

    let mut split_message = message.split_whitespace();
    match split_message.next().unwrap() {
        "POST" => {
            Ok(post_command(message))
        }
        "GET" => {
            Ok(get_command(message))
        }
        _ => {
            Err(Error::new(ErrorKind::Other, "This was not a POST or GET command."))
        }
    }

}

// NOT TESTED
fn post_command(message: String) -> String {
    // This function just inserts the message into the database
    // and then it returns a message to mark success.

    let mut message = messages::parse_message(message);
    message.insert();
    String::from("Message saved successfully.")
}

// NOT TESTED
fn get_command(message: String) -> String {
    // This function right now gets all messages that have both the from and to user
    // involved.

    let mut message = message.split_whitespace();
    message.next();
    let from_user = String::from(message.next().unwrap());
    let mut mes = messages::get_conversation(from_user)
        .expect("There is a problem with get_conversation().");
    if mes == String::new() {
        mes = String::from("null");
    }
    mes
}