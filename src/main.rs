#![allow(dead_code)]

extern crate regex;
extern crate rustc_serialize;
use rustc_serialize::json;

use std::vec::Vec;

mod bspwm;
mod subprogram;
mod json_socket;
mod messages;

use messages::{Command, CommandResponse};


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

#[derive(Debug)]
enum FocusDirection
{
    Next,
    Prev
}
/**
    The result of a focus by direction command. Returns inner if the node is part
    of the stack, outer if not
 */
#[derive(Debug)]
enum FocusResult
{
    Inner,
    Outer
}
impl StackState
{
    pub fn focus_node_by_direction(&self, direction: &FocusDirection) -> FocusResult
    {
        let focused_id = 
                bspwm::get_node_id(&bspwm::get_node_json(bspwm::get_focused_node()));

        //If the currently focused node is outside the stack, we don't do anything
        let focused_index = match self.nodes.binary_search(&focused_id)
        {
            Ok(val) => val,
            Err(_) =>
            {
                return FocusResult::Outer
            }
        };

        //Calculating the target index
        let target_index = focused_index as i64 + match *direction{
            FocusDirection::Next => 1,
            FocusDirection::Prev => -1
        };

        //Focusing the node if it is inside the stack
        if target_index >= 0 && target_index < self.nodes.len() as i64
        {
            self.focus_node_by_index(target_index as usize);
            FocusResult::Inner
        }
        else
        {
            FocusResult::Outer
        }
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
        let root_json = &bspwm::get_node_json(self.root);

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
        bspwm::node_focus(id);
    }
}

/**
    Converts from cardinal directions (north, south, west, east) to FocusDirections based on 
    a split direction
 */
fn cardinal_to_focus_direction(cardinal: &bspwm::CardinalDirection, split: &bspwm::SplitDirection) 
    -> Option<FocusDirection>
{
    match *split
    {
        bspwm::SplitDirection::Horizontal => 
        {
            match *cardinal
            {
                bspwm::CardinalDirection::North => Some(FocusDirection::Prev),
                bspwm::CardinalDirection::South => Some(FocusDirection::Next),
                _ => None
            }
        }
        bspwm::SplitDirection::Vertical => 
        {
            match *cardinal
            {
                bspwm::CardinalDirection::West => Some(FocusDirection::Prev),
                bspwm::CardinalDirection::East => Some(FocusDirection::Next),
                _ => None
            }
        }
    }
}

fn main() 
{
    let focused_json = bspwm::get_node_json(bspwm::get_focused_node());
    let mut stack = create_new_stack(&focused_json);

    let command_handler = |command: Command|
    {
        match command
        {
            Command::CreateStack => {
                stack = create_new_stack(&bspwm::get_node_json(bspwm::get_focused_node()));
                println!("creating stack rooted at {}", stack.root);
                CommandResponse::Done
            },
            Command::IsFocusedInStack => {
                println!("Query for focused");
                CommandResponse::Yes
            }
            Command::Move(direction) => {
                println!("Asked to move in direction: {:?}", direction);

                let real_direction = cardinal_to_focus_direction(&direction, &stack.direction);

                match real_direction
                {
                    Some(dir) => {
                        stack.focus_node_by_direction(&dir);
                        println!("Moving {:?}", dir);
                    },
                    None => {}
                }
                //stack.focus_node_by_index(1);
                CommandResponse::Done
            }
        }
    };

    json_socket::run_read_reply_server(9232, command_handler).unwrap();
}





