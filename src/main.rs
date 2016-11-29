#![allow(dead_code)]

extern crate regex;
use regex::Regex;

extern crate rustc_serialize;
use rustc_serialize::json;

use std::process::Command;

use std::string::String;
use std::vec::Vec;



/**
    Calls a system program with the specified arguments as a vector

    Returns the output as UTF8 if successfull, Err(String) if not
 */
fn call_program(program_name: &str, args: &Vec<&str>) -> Result<String, String>
{
    let mut cmd = Command::new(&program_name);
    
    for arg in args
    {
        cmd.arg(&arg);
    }

    match cmd.output()
    {
        Ok(result) => Ok(String::from_utf8(result.stdout).unwrap()),
        Err(_) => Err("failed to run program".to_string())
    }
}


////////////////////////////////////////////////////////////////////////////////
//                         Bspc calls
////////////////////////////////////////////////////////////////////////////////

/**
    Runs bspc query -N -n $selector
*/
fn node_query(selector: &str) -> Option<Vec<String>>
{
    //Bspc is weird and interprets the query "" as something other than no parameters
    let mut arguments = vec!("query", "-N", "-n");
    if selector.len() != 0
    {
        arguments.push(selector);
    }

    //Actualy run the query
    let node_string = call_program("bspc", &arguments).unwrap();

    //Regex for checking if a string contains nodes
    let query_check = Regex::new(r"0x[0123456789ABCDEF]*").unwrap();

    if query_check.find(&node_string) != None || node_string == "".to_string()
    {
        Some(node_string.split("\n")
                .map( |s| //Convert the &strs to Strings
                {
                    String::from(s)
                })
                .filter(|s| //BSPWM places a \n after all results which results in a trailing result
                {
                    s.len() != 0
                })
                .collect())
    }
    else
    {
        println!("Node query failed, query returned {}", node_string);
        None
    }
}


#[derive(PartialEq, Eq, Debug)]
enum ResizeDirection
{
    Top,
    Left,
    Bottom,
    Right
}
/**
    Tells BSPWM to resize the specified node
*/
fn node_resize(node: &str, direction: &ResizeDirection, amount: i32)
{
    let (direction_str, dx, dy) = match *direction
    {
        ResizeDirection::Top => ("top", 0, -amount),
        ResizeDirection::Bottom => ("bottom", 0, amount),
        ResizeDirection::Left => ("left", 0, -amount),
        ResizeDirection::Right => ("right", 0, amount)
    };

    let program_output = call_program(
        "bspc",
        &vec!(
            "node",
            &node,
            "-z",
            direction_str,
            &format!("{}", dx),
            &format!("{}", dy)
            )
        );

    println!("{}", program_output.unwrap());
}

/**
    Focuses on a specified node
*/
fn node_focus(node: &str)
{
    println!("{}", call_program("bspc", &vec!("node", "-f", node)).unwrap());
}

/**
    Returns the first node in a list of nodes. Performs 2 uwnwraps
    so it will crash if the list is empty or None
*/
fn first_node(list: Option<Vec<String>>) -> String
{
    list.unwrap().pop().unwrap()
}

/**
    Returns the root node
*/
fn get_root_node() -> String
{
    first_node(node_query("@/"))
}

/**
    Querys bspc for the currently focused node
*/
fn get_focused_node() -> String
{
    node_query("").unwrap().pop().unwrap()
}

/**
    Gets the subtree of node as a JSON object
*/
fn get_node_json(node: &str) -> json::Object
{
    let str_json = call_program("bspc", &vec!("query", "-T", "-n", node)).unwrap();

    json::Json::from_str(&str_json).unwrap().as_object().unwrap().clone()
}

#[derive(PartialEq, Clone, RustcEncodable)]
enum SplitDirection
{
    Horizontal,
    Vertical
}
/**
    Returns the split type of a node.
*/
fn get_node_split_direction(node_json: &json::Object) -> SplitDirection
{
    match node_json.get("splitType").unwrap().as_string().unwrap().as_ref()
    {
        "horizontal" => SplitDirection::Horizontal,
        "vertical" => SplitDirection::Vertical,
        _ => panic!("Unknown splitType value")
    }
}

/**
    Returns all children of a specific node
*/
fn get_node_children(node_json: &json::Object) -> Option<(json::Object, json::Object)>
{
    //Try getting the first child field. This should always exist as long as the
    //node is valid
    let first_child = node_json.get("firstChild").unwrap().as_object();

    //Second child can not be None if First childe is Some because the whole
    //tree is a full binary tree
    match first_child
    {
        Some(child) =>
            Some(
                (
                    child.clone(), 
                    node_json.get("secondChild").unwrap().as_object().unwrap().clone()
                )
            ),
        None => None
    }
}

/**
    Returns a list of nodes that can be stacked in the current root node
    
    It will traverse the tree until it either finds a leaf node, or a node that 
    is split the oposite direction of the stack
*/
fn find_target_stack(root: &json::Object, direction: &SplitDirection) -> Vec<u64>
{
    //Check if the node has children
    let node_children = get_node_children(root);

    if node_children == None || get_node_split_direction(&root) != *direction
    {
        vec!(get_node_id(&root))
    }
    else
    {
        let (first_child, second_child) = node_children.unwrap();
        let mut result = find_target_stack(&first_child, direction);
        result.append(&mut find_target_stack(&second_child, direction));
        
        result
    }
}

/**
    Creates a new stack with all the child nodes of the currently focused node
*/
fn create_new_stack(root_node_json: &json::Object) -> StackState
{
    let direction = &get_node_split_direction(&root_node_json);
    let node_list = find_target_stack(&root_node_json, direction);

    StackState{
        direction: direction.clone(),
        nodes: node_list,
        root: get_node_id(&root_node_json)
    }
}

/**
    Returns the ID of a given node from its json representation
*/
fn get_node_id(node_json: &json::Object) -> u64
{
    node_json.get("id").unwrap().as_u64().unwrap()
}


/**
    Returns the a string representation of the ID of a node that can be interpreted by bspc
    (0x...)
*/
fn get_node_name(id: u64) -> String
{
    format!("0x{:X}", id)
}

#[derive(Debug, Eq, PartialEq)]
enum Children
{
    First,
    Second
}
/**
    Returns the list of directions you have to take from a node
    to a descendant
*/
fn find_path_to_node(root: &json::Object, target: u64) -> Option<Vec<Children>>
{
    let node_children = get_node_children(root);
    if get_node_id(root) == target
    {
        return Some(vec!())
    }
    else if node_children == None
    {
        return None
    }
    else
    {
        let (first_child, second_child) = node_children.unwrap();
        let direction_to_first = find_path_to_node(&first_child, target);
        let direction_to_second = find_path_to_node(&second_child, target);

        match (direction_to_first, direction_to_second)
        {
            (None, None) => None,
            (Some(mut path), _) => {
                path.insert(0, Children::First);
                return Some(path);
            },
            (_, Some(mut path)) => {
                path.insert(0, Children::Second);
                return Some(path);
            }
        }
    }
}


/**
    Counts the amount of descendants that a node has
*/
fn count_node_descendant_leaves(root: &json::Object) -> u64
{
    let children = get_node_children(root);

    match children
    {
        None => 1,
        Some(children) =>
        {
            //Split the children tuple
            let (first,second) = children;

            count_node_descendant_leaves(&first) + count_node_descendant_leaves(&second)
        }
    }
}

const SMALL_NODE_SIZE: u64 = 30;

/**
    Struct that keeps track of a window stack
*/
#[derive(Clone, RustcEncodable)]
struct StackState 
{
    pub direction: SplitDirection,
    pub nodes: Vec<u64>,
    pub root: u64
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
        let focused_id = get_node_id(&get_node_json(&get_focused_node()));

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


        fn recursion_helper(
                node_json: &json::Object,
                mut remaining_path: Vec<Children>,
                resize_directions: (SplitDirection, SplitDirection)
            )
        {
            let current_intersection = remaining_path.pop();


            //Find which node should be traversed and which node should be 
            let (should_balance_first, traverse_node, balance_node) = match current_intersection
            {
                None => {return},
                Some(val) => 
                {
                    match val
                    {
                        Children::First =>
                        {
                            let (first_child, second_child) = get_node_children(node_json).unwrap();
                            (false, first_child, second_child)
                        },
                        Children::Second =>
                        {
                            let (first_child, second_child) = get_node_children(node_json).unwrap();
                            (true, second_child, first_child)
                        }
                    }
                }
            };

            //Balance the node that doesn't contain the target
            let node_size = count_node_descendant_leaves(&balance_node) * SMALL_NODE_SIZE;

            let balance_node_name = &get_node_name(&get_node_id(&balance_node));

            //If the first child should be resized, the bottom should be dragged,
            //otherwise the top
            let resize_direction = match should_balance_first
            {
                true => resize_directions.0,
                false => resize_directions.1
            };

            //Set the node to a tiny size
            node_resize(balance_node_name, resize_direction, -1000);
            node_resize(balance_node_name, resize_direction, node_size);

            //Dig deeper
            recursion_helper(&traverse_node, remaining_path);
        };

        let target_node_name = &get_node_name(self.root);
        let root_json = &get_node_json(target_node_name);

        //Finding the 'path' to the target node
        let path = find_path_to_node(root_json, id);

        //Getting the correct directions for the resizing
        let resize_directions = match self.direction
        {
            SplitDirection::Horizontal => (ResizeDirection::Top, ResizeDirection::Bottom),
            _ => (ResizeDirection::Right, ResizeDirection::Left)
        };

        recursion_helper(root_json, path.unwrap(), resize_directions);

        //Focus the actual node
        node_focus(target_node_name);
    }
}


fn main() 
{
    let focused_json = get_node_json(&get_focused_node());
    let stack = create_new_stack(&focused_json);

    stack.focus_node_by_index(3);
}





//Loading some modules that are needed for testing
#[cfg(test)]
mod tests
{
    use super::{
        call_program,
        node_query,
        get_node_children,
        get_node_id,
        find_target_stack,
        SplitDirection,
        find_path_to_node,
        Children,
        count_node_descendant_leaves
    };

    use std::io::prelude::*;
    use std::fs::File;

    use rustc_serialize::json;

    #[test]
    fn basic_call_test()
    {
        assert_eq!(call_program("sh", &vec!("-c", "echo hello")).unwrap(), "hello\n".to_string());

        assert!(call_program("yoloswagmannen", &vec!("-c", "echo hello")).is_err());
    }

    #[test]
    fn bspc_test()
    {
        //Ensure exactly one focused window is returned
        {
            let query_result = node_query("");

            assert!(query_result.is_some());
            
            println!("{:?}", query_result.clone().unwrap());
            assert!(query_result.unwrap().len() == 1);
        }
    }



    #[test]
    fn tree_traversal_test()
    {
        //Load some sample json from a file
        let mut f = File::open("sample_tree.json").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();


        let string_json = json::Json::from_str(&s).unwrap();
        let data = string_json.as_object().unwrap();

        assert!(get_node_children(&data).is_some());
        assert!(get_node_id(&get_node_children(&data).unwrap().0) == 29475921);
        assert!(get_node_id(&get_node_children(&data).unwrap().1) == 4194628);

        let stack_test = find_target_stack(&data, &SplitDirection::Vertical);

        let desired_ids = vec!(29475921, 4194628);
        assert_eq!(stack_test, desired_ids);


        //Testing 'pathfinding'
        assert_eq!(
                find_path_to_node(&data, 29526298),
                Some(vec!(Children::Second, Children::Second, Children::First))
            );
        assert_eq!(
                find_path_to_node(&data, 0),
                None
            );

        assert_eq!(count_node_descendant_leaves(&data), 6);
    }
}
