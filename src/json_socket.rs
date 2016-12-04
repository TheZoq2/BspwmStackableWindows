/*!
    Module for sending and receiving json serialised classes over sockets
*/

use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use rustc_serialize::{json, Encodable, Decodable};

use std::io;

use std::string::String;

pub trait SendableMessage<T> where T: Encodable + Decodable
{
    fn get_reply(&self) -> T;
}

#[derive(RustcEncodable, RustcDecodable, Eq, PartialEq)]
pub struct TestResponse
{
    x: i32
}

#[derive(RustcEncodable, RustcDecodable)]
pub struct TestMessage
{
    x: i32
}

impl SendableMessage<TestResponse> for TestMessage
{
    fn get_reply(&self) -> TestResponse 
    {
        TestResponse{x: self.x * 2}
    }
}


/**
    Connects to a remote tcp socket, sends a message, waits for a reply
    and returns that reply
*/
pub fn connect_send_read<MsgType, ResponseType>(address: &str, msg: MsgType) 
                -> io::Result<ResponseType>
    where MsgType: SendableMessage<ResponseType> + Encodable + Decodable, 
            ResponseType: Decodable + Encodable
{
    let mut stream = TcpStream::connect(address)?;

    //Encode the message as json
    let encoded = json::encode(&msg).unwrap();
    let encoded_as_string = encoded.to_string();

    //stream.write_all(encoded_as_string.to_bytes());
    unimplemented!();
}

/**
    Returns the reply to a given message
 */
pub fn get_reply_to_message<ReplyType, MsgType>(msg: MsgType) -> ReplyType
    where MsgType: SendableMessage<ReplyType> + Encodable + Decodable, 
        ReplyType: Encodable + Decodable
{
    msg.get_reply()
}


#[cfg(test)]
mod json_socket_tests
{
    use super::*;

    #[test]
    fn woodo_tests() 
    {
        let msg = TestMessage{x: 5};

        assert_eq!(get_reply_to_message(msg).x, 10);
    }
}


/**
    Starts a TCP listener that waits for incomming connections, read what they have to say
    and return a message
*/
pub fn start_tcp_listener(port: u16)
{
    
}
