/*!
    Module for sending and receiving json serialised classes over sockets
*/

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use rustc_serialize::{json, Encodable, Decodable};

use std::io;

use std::string::String;

struct JsonMessage<MessageType, PayloadType>
    where MessageType: Encodable + Decodable + Eq + PartialEq, PayloadType: Encodable + Decodable
{
    message_type: MessageType,
    payload: PayloadType
}

/**
    Connects to a remote tcp socket, sends a message, waits for a reply
    and returns that reply
*/
pub fn connect_send_read<MsgType, ResponseType>(address: &str, msg: )
                -> io::Result<ResponseType>
    where MsgType: Encodable+Decodable, ResponseType: Encodable+Decodable
{
    let mut stream = TcpStream::connect(address)?;

    //Encode the message as json
    let encoded = json::encode(&msg).unwrap();
    let encoded_as_string = encoded.to_string();

    stream.write_all(&encoded_as_string.into_bytes()).unwrap();

    let mut response_buffer = vec!();
    stream.read_to_end(&mut response_buffer).unwrap();

    //Convert the response to a string and decode it
    let response_as_string = String::from_utf8(response_buffer).unwrap();
    let decoded: ResponseType = json::decode(&response_as_string).unwrap();

    Ok(decoded)
}


/**
    Starts a TCP listener that waits for incomming connections, read what they have to say
    and return a message
*/
pub fn start_tcp_listener(port: u16)
{
    
}
