/*!
    Module for sending and receiving json serialised classes over sockets
*/

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use rustc_serialize::{json, Encodable, Decodable};

use std::io;

use std::string::String;


/**
    Connects to a remote tcp socket, sends a message, waits for a reply
    and returns that reply
*/
fn connect_send_read<MsgType, ResponseType>(address: &str, msg: MsgType) 
                -> io::Result<ResponseType>
    where MsgType: Encodable, ResponseType: Decodable
{
    let mut stream = TcpStream::connect(address)?;

    //Encode the message as json
    let encoded = json::encode(&msg).unwrap();
    let encoded_as_string = encoded.to_string();

    stream.write_all(encoded_as_string.to_bytes());
}
