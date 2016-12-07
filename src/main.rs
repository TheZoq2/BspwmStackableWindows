#![allow(dead_code)]

extern crate regex;
extern crate rustc_serialize;
use rustc_serialize::json;

use std::vec::Vec;

mod bspwm;
mod subprogram;
mod json_socket;


////////////////////////////////////////////////////////////////////////////////
//                         Bspc calls
////////////////////////////////////////////////////////////////////////////////


/**
    Struct that keeps track of a window stack
*/
#[derive(Clone, RustcEncodable)]
struct StackState 
{
    pub direction: bspwm::SplitDirection,
    pub nodes: Vec<u64>,
    pub root: u64
}

/**
    Creates a new stack with all the child nodes of the currently focused node
*/
fn create_new_stack(root_node_json: &json::Object) -> StackState
{
    let direction = &bspwm::get_node_split_direction(&root_node_json);
    let node_list = bspwm::find_target_stack(&root_node_json, direction);

    StackState{
        direction: direction.clone(),
        nodes: node_list,
        root: bspwm::get_node_id(&root_node_json)
    }
}

enum FocusDirection
{
    Next,
    Prev
}
impl StackState
{
    pub fn focus_next_node(&self, direction: &FocusDirection)
    {
        let focused_id = 
                bspwm::get_node_id(&bspwm::get_node_json(&bspwm::get_focused_node()));

        //We assume the focused id is in the stack
        let focused_index = self.nodes.binary_search(&focused_id).unwrap();

        let target_index = focused_index as i64 + match *direction{
            FocusDirection::Next => 1,
            FocusDirection::Prev => -1
        };

        //Calculating the final index. +self.nodes.len() .. % allows wrap around when the
        //target index is negative
        let final_index = (target_index + self.nodes.len() as i64) % self.nodes.len() as i64;

        self.focus_node_by_index(final_index as usize);
    }

    /**
        Focus a node with the specified index in the stack. 
    */
    pub fn focus_node_by_index(&self, index: usize)
    {
        let target_node = self.nodes[index as usize];

        self.focus_node_by_id(target_node);
    }

    fn focus_node_by_id(&self, id: u64)
    {
        let target_node_name = &bspwm::get_node_name(self.root);
        let root_json = &bspwm::get_node_json(target_node_name);

        //Finding the 'path' to the target node
        let path = bspwm::find_path_to_node(root_json, id);

        //Getting the correct directions for the resizing
        let resize_directions = match self.direction
        {
            bspwm::SplitDirection::Horizontal => 
            {
                (bspwm::ResizeDirection::Top, bspwm::ResizeDirection::Bottom)
            },
            _ => 
            {
                (bspwm::ResizeDirection::Right, bspwm::ResizeDirection::Left)
            }
        };

        bspwm::focus_node_by_path(root_json, path.unwrap(), &resize_directions);

        //Focus the actual node
        bspwm::node_focus(target_node_name);
    }

}


////////////////////////////////////////////////////////////////////////////////
//                          Networking stuff
////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, RustcEncodable, RustcDecodable)]
enum Commands
{
    CreateStack,
    IsFocusedInStack,
    Move(bspwm::Direction)
}
#[derive(Debug, RustcEncodable, RustcDecodable)]
enum CommandResponse
{
    Done,
    NoStackExists,
    EndOfStack,
    Yes,
    No
}

fn main() 
{
    let focused_json = bspwm::get_node_json(&bspwm::get_focused_node());
    let mut stack = create_new_stack(&focused_json);

    let command_handler = |command: Commands|
    {
        match command
        {
            Commands::CreateStack => {
                println!("Creating a stack");
                CommandResponse::Done
            },
            Commands::IsFocusedInStack => {
                println!("Query for focused");
                stack.focus_next_node(&FocusDirection::Next);
                CommandResponse::Yes
            }
            Commands::Move(direction) => {
                println!("Asked to move in direction: {:?}", direction);
                CommandResponse::Done
            }
        }
    };

    json_socket::run_read_reply_server(9232, command_handler).unwrap();
}





