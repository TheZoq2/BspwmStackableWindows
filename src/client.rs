#![allow(dead_code)]

#[macro_use]
extern crate clap;

extern crate regex;
extern crate rustc_serialize;

mod bspwm;
mod subprogram;
mod json_socket;
mod messages;

use messages::{Commands, CommandResponse};

pub fn main()
{
    let matches = clap_app!(myapp =>
            (version: "1.0")
            (about: "Does awesome things")
            (@arg CONFIG: -c --config +takes_value "Sets a custom config file")
            (@arg INPUT: +required "Sets the input file to use")
            (@arg debug: -d ... "Sets the level of debugging information")
            (@subcommand test =>
                (about: "controls testing features")
                (version: "1.3")
                (author: "Someone E. <someone_else@other.com>")
                (@arg verbose: -v --verbose "Print test information verbosely")
            )
        ).get_matches();
}
