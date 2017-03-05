#![allow(dead_code)]

extern crate clap;

extern crate regex;
extern crate rustc_serialize;
extern crate typed_messages;

mod bspwm;
mod subprogram;
mod messages;

use bspwm::{FocusTarget, CardinalDirection};

use clap::{App, Arg, SubCommand};

pub fn focus_neighbour(
        neighbour: Option<u64>,
        current_desktop_nodes: Vec<u64>,
        neighbour_desktop_nodes: Vec<u64>,
        neighbour_desktop: Option<u64>,
    )
    -> FocusTarget
{
    match neighbour
    {
        Some(neighbour_id) =>
        {
            if neighbour_desktop_nodes.contains(&neighbour_id)
                || current_desktop_nodes.contains(&neighbour_id)
            {
                FocusTarget::Node(neighbour_id)
            }
            else
            {
                FocusTarget::Desktop(neighbour_desktop.unwrap())
            }
        }
        None => {
            match neighbour_desktop
            {
                Some(neighbour_desktop) => FocusTarget::Desktop(neighbour_desktop),
                None => FocusTarget::None
            }
        }
    }
}

pub fn focus_direction(direction: CardinalDirection)
{
    let current_node = bspwm::get_focused_node();

    let neighbour = match current_node
    {
        Some(node) => bspwm::get_neighbouring_node(node, direction),
        None => None
    };

}

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

        let direction = CardinalDirection::from_str(direction).unwrap();
        println!("{:?}", direction);

        println!("{:?}", bspwm::get_focused_node());
    }
    else
    {
        println!("No subcommand specified");
    }
}

#[cfg(test)]
mod focus_tests
{
    use super::*;

    #[test]
    fn empty_desktop()
    {
        let desktop_id = 5;
        let target = focus_neighbour(None, vec!(), vec!(), Some(desktop_id));
        assert_eq!(target, FocusTarget::Desktop(desktop_id));

        let target = focus_neighbour(None, vec!(), vec!(10,20), Some(desktop_id));
        assert_eq!(target, FocusTarget::Desktop(desktop_id));
    }

    #[test]
    fn same_desktop()
    {
        let node_id = 5;
        let target = focus_neighbour(Some(node_id), vec!(node_id, 4, 3), vec!(), None);

        assert_eq!(target, FocusTarget::Node(node_id));
    }

    #[test]
    fn node_in_other_desktop()
    {
        let node_id = 5;
        let target = focus_neighbour(Some(node_id), vec!(3,2), vec!(node_id, 4,1), None);
        assert_eq!(target, FocusTarget::Node(node_id));
    }

    #[test]
    fn node_in_faraway_desktop()
    {
        let desktop_id = 5;

        let target = focus_neighbour(Some(2), vec!(1259, 19251), vec!(12912,12612), Some(desktop_id));
        assert_eq!(target, FocusTarget::Desktop(desktop_id));
    }

    #[test]
    fn no_neighbour()
    {
        let target = focus_neighbour(None, vec!(1925, 15212), vec!(), None);

        assert_eq!(target, FocusTarget::None);
    }
}
