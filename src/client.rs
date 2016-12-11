#![allow(dead_code)]

extern crate clap;

extern crate regex;
extern crate rustc_serialize;

mod bspwm;
mod subprogram;
mod json_socket;
mod messages;

use messages::{Command, CommandResponse};

use json_socket::connect_send_read;

use clap::{App, Arg, SubCommand};

fn try_send_message(command: Command) -> Option<CommandResponse>
{
    match connect_send_read::<_, CommandResponse>("localhost", 9232, command)
    {
        Ok(result) => Some(result),
        Err(e) => {
            println!("Failed to send message. Error: {:?}", e);
            None
        }
    }
}

fn do_create_stack()
{
    let response = try_send_message(Command::CreateStack);

    match response
    {
        Some(CommandResponse::Done) => {
            println!("Stack created sucessfully");
        },
        Some(other) => {
            println!("Server replied unexpectedly to create stack request. Expected Done, got {:?}", other);
        }
        None => {}
    }
}

pub fn main()
{
    let stack_subcommand = SubCommand::with_name("stack")
        .about("controls stacks")
        .arg(Arg::with_name("create")
            .help("Create a new stack rooted at the currently focused node"));

    let arg_parser = App::new("stack_client")
        .about("Client for bspwm stackable windows")
        .arg(Arg::with_name("arg1"))
        .subcommand(stack_subcommand);

    let matches = arg_parser.get_matches();

    if let Some(matches) = matches.subcommand_matches("stack")
    {
        if matches.is_present("create")
        {
            do_create_stack();
        }
        else
        {
            println!("Invalid subcommand");
        }
    }
    else
    {
        println!("No subcommand specified");
    }
}
