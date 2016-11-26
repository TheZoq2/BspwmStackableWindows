#![allow(dead_code)]

extern crate regex;
use regex::Regex;

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
    Returns the children of a specified node. 
    
    If children exist, the first and second child are returned as a tuple, otherwise
    None is returned
*/
fn get_children(root: &str) -> Option<(String, String)>
{
    let first_query_string = root.clone().to_string() + "#@1";
    let second_query_string = root.clone().to_string() + "#@2";

    let mut first_child = node_query(&first_query_string).unwrap();
    let mut second_child = node_query(&second_query_string).unwrap();

    if first_child.len() == 0
    {
        None
    }
    else
    {
        Some((
                first_child.pop().unwrap(),
                second_child.pop().unwrap()
            ))
    }
}

/**
    Returns all the children of a specified node
*/
fn get_descendant_leaves(root: &str) -> Vec<String>
{
    let children = get_children(root);

    //let mut result = vec!(root.to_string());
    let mut result = vec!();

    match children
    {
        None => {result.push(root.to_string())},
        Some((left_child, right_child)) => {
            println!("{} {}", left_child, right_child);

            result.append(&mut get_descendant_leaves(&left_child));
            result.append(&mut get_descendant_leaves(&right_child));
        }
    }

    result
}


fn main() 
{
    node_resize(&get_focused_node(), ResizeDirection::Top, 0, 100);
}





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

    //Ensure that a root is returned
    {
        let query_result = node_query("@/");

        assert!(query_result.is_some());

        println!("{:?}", query_result.clone().unwrap());
        assert!(query_result.unwrap().len() == 1);
    }
}
