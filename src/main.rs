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
    /**
      Creates a new stack containing all the child nodes of the current focused
      node
    */
    pub fn new(root_json: &json::Object) -> StackState
    {
        let direction = &bspwm::get_node_split_direction(&root_json);
        let node_list = bspwm::find_target_stack(&root_json, direction);

        StackState{
            direction: direction.clone(),
            nodes: node_list,
            root: bspwm::get_node_id(&root_json)
        }
    }

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

    /**
      Makes the speicifed node focused if it is part of the stack. If not,
      do nothing

      TODO: If a parent node of the specified node is part of the stack, focus
      it  instead
    */
    fn focus_node_by_id(&self, id: u64)
    {
        let root_json = &bspwm::get_node_json(self.root);

        //Finding the 'path' to the target node
        let path = bspwm::find_path_to_node(root_json, id);

        if path.is_none()
        {
            return
        }

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

    fn contains_node(&self, id: u64) -> bool
    {
        let root_json = bspwm::get_node_json(self.root);

        let descendants = bspwm::get_node_descendants(&root_json);
        println!("Looking for node {} in {:?}", id, descendants);
        descendants.contains(&id)
    }

    fn cleanup(&self)
    {
        bspwm::node_balance(self.root);
    }
}

/**
    Converts from cardinal directions (north, south, west, east) to FocusDirections based on 
    a split direction
 */
fn cardinal_to_focus_direction
            (cardinal: &bspwm::CardinalDirection, split: &bspwm::SplitDirection) 
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

fn remove_stack_containing_node(stack_vec: &mut Vec<StackState>, id: u64) -> CommandResponse
{
    // Find all the stacks that contain the current node as a child.
    // The stacks are stored as (index, stack) to all
    let target_index = 
    {
        let (_, mut matching_stacks) = stack_vec.iter()
            .fold(
                (0, vec!()),
                |accumulator, stack|
                {
                    let (index, mut stacks) = accumulator;
                    if stack.contains_node(id)
                    {
                        stacks.push((index, stack));
                    }
                    (index + 1, stacks)
                }
            );

        if matching_stacks.len() > 0
        {
            //Check which stack is at the bottom of all the stacks
            //that contain the actual target
            let first_match = matching_stacks.pop().unwrap();

            let target = matching_stacks.into_iter().fold(
                first_match, 
                |best_match, stack|
                {
                    let (index, stack) = stack;
                    let (best_index, best_stack) = best_match;

                    match best_stack.contains_node(stack.root)
                    {
                        true => (index, stack),
                        false => (best_index, best_stack)
                    }
                });

            let (target_index, _) = target;
            Some(target_index)
        }
        else
        {
            println!("No stack removed");
            None
        }
    };

    if target_index.is_some()
    {
        let target_index = target_index.unwrap();

        stack_vec[target_index].cleanup();
        stack_vec.remove(target_index);
        CommandResponse::Done
    }
    else
    {
        CommandResponse::NoStackExists
    }
}

fn is_node_in_stacks(stacks: &Vec<StackState>, node: u64) -> bool
{
    stacks.iter()
        .fold(false, |acc, stack|{acc || stack.contains_node(node)})
}

fn main() 
{
    let mut stacks = vec!();

    let command_handler = |command: Command|
    {
        match command
        {
            Command::CreateStack => {
                let stack = StackState::new(&bspwm::get_node_json(bspwm::get_focused_node()));
                println!("creating stack rooted at {}", stack.root);
                stacks.push(stack);

                CommandResponse::Done
            },
            Command::RemoveFocused => {
                let focused = bspwm::get_focused_node();

                remove_stack_containing_node(&mut stacks, focused)
            },
            Command::IsFocusedInStack => {
                let focused = bspwm::get_focused_node();

                match is_node_in_stacks(&stacks, focused)
                {
                    true => CommandResponse::Yes,
                    false => CommandResponse::No
                }
            },
            Command::FocusCurrent => {
                for stack in &stacks
                {
                    stack.focus_node_by_id(bspwm::get_focused_node())
                }

                CommandResponse::Done
            }
            Command::UpdateStacks => {
                CommandResponse::Done
            }
        }
    };

    json_socket::run_read_reply_server(9232, command_handler).unwrap();
}






