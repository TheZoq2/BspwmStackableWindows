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


fn direction_from_string(string: &str) -> Result<bspwm::CardinalDirection, String>
{
    match string.to_lowercase().as_str()
    {
        "north" => Ok(bspwm::CardinalDirection::North),
        "south" => Ok(bspwm::CardinalDirection::South),
        "east"  => Ok(bspwm::CardinalDirection::East),
        "west"  => Ok(bspwm::CardinalDirection::West),
        other => Err(String::from(other))
    }
}

/**
    Tries to send a message to the server
 */
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

/**
    Handles responses from the server where the expected output is Command::Done
 */
fn handle_done_fail_response(response: Option<CommandResponse>, ok_msg: &str)
{
    match response
    {
        Some(CommandResponse::Done) => {
            println!("{}", ok_msg);
        },
        Some(other) => {
            println!("Server replied unexpectedly. Expected Done, got {:?}", other);
        }
        None => {}
    }
}

fn do_create_stack()
{
    let response = try_send_message(Command::CreateStack);

    handle_done_fail_response(response, "Stack created successfully");
}

fn do_move(direction: bspwm::CardinalDirection)
{
    let response = try_send_message(Command::Move(direction));

    handle_done_fail_response(response, "Move complete");
}

fn do_focus_current()
{
    let response = try_send_message(Command::FocusCurrent);

    handle_done_fail_response(response, "Current is focused")
}


fn handle_stack_move(subcommand: Option<&str>)
{
    match subcommand
    {
        Some(direction) =>
        {
            match direction_from_string(direction)
            {
                Ok(val) => do_move(val),
                Err(val) => println!("Invalid move direction: {}", val)
            }
        },
        None =>
        {
            println!("Focus requires a direction")
        }
    }
}

pub fn main()
{
    let stack_subcommand = SubCommand::with_name("stack")
        .about("controls stacks")
        .arg(Arg::with_name("command")
            .required(true)
            .help("Primary command. {create, move}"))
        .arg(Arg::with_name("parameters")
            .help("Additional parameters to the comand"));

    let arg_parser = App::new("stack_client")
        .about("Client for bspwm stackable windows")
        .subcommand(stack_subcommand);

    let matches = arg_parser.get_matches();

    if let Some(matches) = matches.subcommand_matches("stack")
    {
        if matches.is_present("command")
        {
            //do_create_stack();
            let command = matches.value_of("command").unwrap();

            match command
            {
                "create" => {
                    do_create_stack();
                },
                "focus" => {
                    handle_stack_move(matches.value_of("parameters"))
                },
                "focus_current" => {
                    do_focus_current();
                }
                other => {
                    println!("unexpected stack command: {}", other);
                }
            }
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
