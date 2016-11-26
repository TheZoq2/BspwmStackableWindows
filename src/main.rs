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
fn node_resize(node: &str, direction: ResizeDirection, dx: i32, dy: i32)
{
    let direction_str = match direction
    {
        ResizeDirection::Top => "top",
        ResizeDirection::Left => "left",
        ResizeDirection::Bottom => "bottom",
        ResizeDirection::Right => "right"
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

#[derive(PartialEq)]
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
    Returns the ID of a given node from its json representation
*/
fn get_node_id(node_json: &json::Object) -> u64
{
    node_json.get("id").unwrap().as_u64().unwrap()
}


fn main() 
{
    let focused_json = get_node_json(&get_focused_node());
    println!("{:?}", find_target_stack(&focused_json, &get_node_split_direction(&focused_json)));
}





//Loading some modules that are needed for testing
mod tests
{
    use super::{
        call_program,
        node_query,
        get_node_children,
        get_node_id,
        find_target_stack,
        SplitDirection,
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
    }
}
