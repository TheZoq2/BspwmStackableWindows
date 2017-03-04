#![allow(dead_code)]

extern crate clap;

extern crate regex;
extern crate rustc_serialize;
extern crate typed_messages;

mod bspwm;
mod subprogram;
mod messages;

use clap::{App, Arg, SubCommand};

pub fn main()
{
    let stack_subcommand = SubCommand::with_name("focus")
        .about("Change focus to some different window")
        .arg(Arg::with_name("direction")
            .required(true)
            .help("What direction to change focus {north, south, east, west}"));

    let arg_parser = App::new("stack_client")
        .about("Wrapper around bspwm for increased control of window movement and focus")
        .subcommand(stack_subcommand);

    let matches = arg_parser.get_matches();

    if let Some(matches) = matches.subcommand_matches("focus")
    {
        //do_create_stack();
        let direction = matches.value_of("direction").unwrap();

        let direction = bspwm::CardinalDirection::from_str(direction).unwrap();
        println!("{:?}", direction);

        println!("{:?}", bspwm::get_focused_node());
    }
    else
    {
        println!("No subcommand specified");
    }
}
