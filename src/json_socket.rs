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
pub fn handle_read_reply_client<MsgType, ReplyType, Function, InputType>
                (ref mut reply_handler: Function, stream: &mut InputType)
        -> JsonHandlerResult<()>
    where MsgType: Encodable + Decodable, 
          ReplyType: Encodable + Decodable,
          Function: FnMut(MsgType) -> ReplyType,
          InputType: Read + Write
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
        handle_read_reply_client(&reply_handler, &mut stream?)?;
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

/**
    Sends a message to an IO stream
*/
pub fn send_message_read_reply<MsgType, ReplyType, IOType>(msg: MsgType, io_stream: &mut IOType)
        -> JsonHandlerResult<ReplyType>
    where 
        MsgType: Encodable + Decodable, 
        ReplyType: Encodable + Decodable,
        IOType: Read + Write
{
    //Encode the message as json
    let encoded = json::encode(&msg).unwrap();
    let encoded_as_string = encoded.to_string();

    //Send it through the socket
    io_stream.write_all(&encoded_as_string.into_bytes())?;

    //Wait for a reply
    let mut buffer = String::new();
    io_stream.read_to_string(&mut buffer)?;

    let decoded = json::decode(&buffer)?;
    Ok(decoded)
}




#[cfg(test)]
mod json_socket_tests
{
    use super::*;


    use std::io::{Read, Write};
    use std::io;

    use rustc_serialize::json;

    struct ReaderWriterDummy
    {
        ///Dummy buffer that is read from
        read_buffer: Vec<u8>, 
        ///Dummy buffer that is written to
        write_buffer: Vec<u8>, 
    }

    impl ReaderWriterDummy
    {
        pub fn new(mut read_content: Vec<u8>) -> Self 
        {
            read_content.reverse();

            ReaderWriterDummy {
                read_buffer: read_content,
                write_buffer: vec!()
            }
        }
        fn get_written(&self) -> &Vec<u8>
        {
            &self.write_buffer
        }
    }

    impl Read for ReaderWriterDummy
    {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>
        {
            match self.read_buffer.pop()
            {
                Some(val) => {
                    buf[0] = val;
                    Ok(1)
                },
                None => Ok(0)
            }
        }
    }
    impl Write for ReaderWriterDummy
    {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize>
        {
            for elem in buf
            {
                self.write_buffer.push(*elem);
            }

            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()>
        {
            Ok(())
        }
    }

    #[test]
    fn meta_read_tests()
    {
        {
            let mut dummy = ReaderWriterDummy::new(Vec::from("56".as_bytes()));

            let mut buffer = String::new();
            dummy.read_to_string(&mut buffer).unwrap();
            assert_eq!(buffer, "56");
        }
        {
            let mut dummy = ReaderWriterDummy::new(Vec::from("".as_bytes()));

            let mut buffer = String::new();
            dummy.read_to_string(&mut buffer).unwrap();
            assert_eq!(buffer, "");
        }
    }

    #[test]
    fn meta_write_tests()
    {
        {
            let mut dummy = ReaderWriterDummy::new(vec!());

            let buffer = String::from("yoloswag");
            dummy.write_all(&buffer.into_bytes()).unwrap();

            let written = dummy.get_written().clone();

            println!("{}", written.len());

            assert_eq!(
                    String::from_utf8(written).unwrap(), 
                    String::from("yoloswag")
                );
        }
    }

    #[test]
    fn repl_test()
    {
        {
            let json_encoded = json::encode(&56).unwrap();

            //Create a dummy buffer containing 56
            let mut dummy = ReaderWriterDummy::new(json_encoded.into_bytes());

            assert!(send_message_read_reply::<i32, i32, ReaderWriterDummy>(5, &mut dummy).unwrap() == 56);
        }

        {
            let response_function = |x: i32|{x * x};

            let mut dummy = ReaderWriterDummy::new(json::encode(&10).unwrap().into_bytes());

            assert!(handle_read_reply_client(&response_function, &mut dummy).is_ok());
            assert!(dummy.get_written() == &json::encode(&100).unwrap().into_bytes());
        }

        {
            let mut buffer = 0;
            {
                let response_function = |x|{buffer = x};

                let mut dummy = ReaderWriterDummy::new(json::encode(&10).unwrap().into_bytes());

                handle_read_reply_client(response_function, &mut dummy).is_ok();
            }

            assert!(buffer == 10);
        }
    }
}

