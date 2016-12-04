/*!
    Module for sending and receiving json serialised classes over sockets
*/

use std::net::{TcpStream, TcpListener};

use rustc_serialize::{json, Encodable, Decodable};

use std::{io, result, convert};

use std::io::prelude::*;




#[derive(Debug)]
pub enum JsonHandlerError
{
    ReadFail(io::Error),
    DecoderError(json::DecoderError),
    EncoderError(json::EncoderError)
}
pub type JsonHandlerResult<T> = result::Result<T, JsonHandlerError>;



//TODO: Maybe turn into a macro
impl convert::From<io::Error> for JsonHandlerError
{
    fn from(error: io::Error) -> JsonHandlerError
    {
        JsonHandlerError::ReadFail(error)
    }
}
impl convert::From<json::DecoderError> for JsonHandlerError
{
    fn from(error: json::DecoderError) -> JsonHandlerError
    {
        JsonHandlerError::DecoderError(error)
    }
}
impl convert::From<json::EncoderError> for JsonHandlerError
{
    fn from(error: json::EncoderError) -> JsonHandlerError
    {
        JsonHandlerError::EncoderError(error)
    }
}


/**
  Replies to a single message of MsgType with a message of ReplyType using
  the reply_handler function
*/
pub fn handle_read_reply_client<MsgType, ReplyType, Function>(reply_handler: &Function, mut stream: TcpStream)
        -> JsonHandlerResult<()>
    where MsgType: Encodable + Decodable, 
          ReplyType: Encodable + Decodable,
          Function: Fn(MsgType) -> ReplyType
{
    let mut buffer = String::new();
    stream.read_to_string(&mut buffer)?;

    //Decode the message. If the message is not of the specified type, this fails.
    let decoded = json::decode(&buffer)?;

    //Run the reply handler to get a reply
    let reply = reply_handler(decoded);

    //Encode the result and send it back
    let encoded = json::encode(&reply)?;
    stream.write_all(&encoded.into_bytes())?;

    Ok(())
}

/**
  Creates a TCP listener that listens for messages of a certain type, and replies with messages
  of another type by running the reply_handler on those messages
 */
pub fn run_read_reply_server<MsgType, ReplyType, Function>(port: u16, reply_handler: Function) 
        -> JsonHandlerResult<()>
    where MsgType: Encodable + Decodable, 
          ReplyType: Encodable + Decodable,
          Function: Fn(MsgType) -> ReplyType
{
    let address: &str = &format!("127.0.0.1:{}", port);
    //let tcp_listener = TcpListener::bind(&format!("127.0.0.1:80", port))?;
    let tcp_listener = TcpListener::bind(address)?;

    for stream in tcp_listener.incoming()
    {
        handle_read_reply_client(&reply_handler, stream?)?;
    }

    Ok(())
}



/**
    Connects to a remote tcp socket, sends a message, waits for a reply
    and returns that reply
*/
pub fn connect_send_read<MsgType, ReplyType>(ip: &str, port: u16, msg: MsgType)
        -> JsonHandlerResult<ReplyType>
    where MsgType: Encodable + Decodable, ReplyType: Encodable + Decodable
{
    let address: &str = &format!("{}:{}", ip, port);
    let mut stream = TcpStream::connect(address)?;

    //Encode the message as json
    let encoded = json::encode(&msg).unwrap();
    let encoded_as_string = encoded.to_string();

    //Send it through the socket
    stream.write_all(&encoded_as_string.into_bytes())?;

    //Wait for a reply
    let mut buffer = String::new();
    stream.read_to_string(&mut buffer)?;

    let decoded = json::decode(&buffer)?;
    Ok(decoded)
}




#[cfg(test)]
mod json_socket_tests
{
    use super::*;

    use std::thread;

    #[test]
    fn repl_test()
    {
        let port = 8912;

        let test_handler = |x: i32|{x * 2};

        //If panic happens here, the test will still pass. TODO
        thread::spawn(move ||{
            println!("Server started");

            run_read_reply_server(port, test_handler).unwrap();
        });
        
        //Give some time for the servero to start
        thread::sleep_ms(100);

        assert!(connect_send_read::<i32, i32>("localhost", port, 5).unwrap() == 10);
    }
}
