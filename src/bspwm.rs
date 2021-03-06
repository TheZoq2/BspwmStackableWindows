
extern crate regex;
use regex::Regex;

extern crate rustc_serialize;
use rustc_serialize::json;

use std::string::String;
use std::vec::Vec;

use subprogram::call_program;

////////////////////////////////////////////////////////////////////////////////
//                          Bspwm related datatypes
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FocusTarget
{
    Node(u64),
    Desktop(u64),
    None
}


#[derive(PartialEq, Eq, Debug)]
pub enum ResizeDirection
{
    Top,
    Left,
    Bottom,
    Right
}


#[derive(PartialEq, Clone, RustcEncodable)]
pub enum SplitDirection
{
    Horizontal,
    Vertical
}



#[derive(Debug, Eq, PartialEq)]
pub enum Children
{
    First,
    Second
}




#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum CardinalDirection
{
    North,
    South,
    West,
    East
}

impl CardinalDirection
{
    pub fn from_str(string: &str) -> Option<CardinalDirection>
    {
        match string
        {
            "north" => Some(CardinalDirection::North),
            "south" => Some(CardinalDirection::South),
            "west" => Some(CardinalDirection::West),
            "east" => Some(CardinalDirection::East),
            _ => None
        }
    }

    pub fn as_str(&self) -> &str
    {
        match *self
        {
            CardinalDirection::North => "north",
            CardinalDirection::South => "south",
            CardinalDirection::East => "east",
            CardinalDirection::West => "west"
        }
    }
}




////////////////////////////////////////////////////////////////////////////////
//                  Query and low level bspwm commands
////////////////////////////////////////////////////////////////////////////////

fn general_query(query_type: &str, flags: Vec<(&str, &str)>) -> Result<Vec<u64>, String>
{
    //Bspc is weird and interprets the query "" as something other than no parameters
    let mut arguments = vec!("query", query_type);

    for (flag, selector) in flags
    {
        if flag.len() != 0
        {
            arguments.push(flag);
        }

        if selector.len() != 0
        {
            arguments.push(selector);
        }
    }

    //Actualy run the query
    let node_string = call_program("bspc", &arguments).unwrap();

    //Regex for checking if a string contains nodes
    let query_check = Regex::new(r"0x[0123456789ABCDEF]*").unwrap();

    if query_check.find(&node_string) != None || node_string == "".to_string()
    {
        Ok(node_string.split("\n")
                .filter(|s| //BSPWM places a \n after all results which results in a trailing result
                {
                    s.len() != 0
                })
                .map( |s| //Parse as u64
                {
                    //Remove 0x and parse the rest as a hexadecimal number
                    u64::from_str_radix(&s[2..], 16).unwrap()
                })
                .collect())
    }
    else
    {
        Err(format!("Node query failed, query returned {}", node_string))
    }
}


//TODO: Return error instead of option
/**
  Runs a bspwm node query with the specified selector and optionally the specified flag
  before it

  ```
  bspc query -N <flag> <selector>
  ```
 */
pub fn node_query_with_flag(selector: &str, flag: &str) -> Option<Vec<u64>>
{
    match general_query("-N", vec!((flag, selector)))
    {
        Ok(val) => Some(val),
        Err(msg) => {
            println!("{}", msg);
            None
        }
    }
}


//TODO: Update to use result instead of option
/**
    Runs bspc query -N -n $selector
*/
pub fn node_query(selector: &str) -> Option<Vec<u64>>
{
    node_query_with_flag(selector, "-n")
}


/**
  Runs bspc query -D -d $selector.
  Err if bspwm returns something unexpected
*/
pub fn desktop_query(selector: &str) -> Result<Vec<u64>, String>
{
    general_query("-D", vec!(("-d", selector)))
}


/**
  Tries to get the root node of the specified desktop
*/
pub fn desktop_top_node_query(desktop: u64) -> Result<Option<u64>, String>
{
    let all_nodes = general_query("-N", vec!(("-d", format!("{}", desktop))))?;

    fn inner(node: u64) -> Result<u64, String>
    {
        let new_nodes = match node_query(format!("{}#@parent", node))
        {
            Some(nodes) => Ok(nodes),
            None => Err("No child nodes found for")
        }?;



        // We found the root
        if new_nodes.len() == 0
        {
            node
        }
        else
        {
            inner(new_nodes[0])
        }

    }

    if all_nodes.len() == 0
    {
        Ok(None)
    }
    else
    {
        Ok()
    }
}









////////////////////////////////////////////////////////////////////////////////
//                      state manipulation commands
////////////////////////////////////////////////////////////////////////////////

/**
    Tells BSPWM to resize the specified node
*/
pub fn node_resize(node: &str, direction: &ResizeDirection, amount: i32)
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

    println!("Node resize output: {}", program_output.unwrap());
}

/**
    Sets the ratio between the first and second node
*/
pub fn node_change_ratio(node: &str, new_ratio: f32)
{
    let _ = call_program(
        "bspc",
        &vec!(
            "node",
            &node,
            "-r",
            &format!("{}", new_ratio)
            )
        );

    //println!("{}", program_output.unwrap());
}

/**
    Balances the children in the specified node
*/
pub fn node_balance(node: u64)
{
    let node_name = get_node_name(node);

    call_program(
        "bspc",
        &vec!(
            "node",
            &node_name,
            "-B",
            )
        ).unwrap();
}

/**
    Focuses on a specified node
*/
pub fn node_focus(node: u64)
{
    call_program("bspc", &vec!("node", "-f", &format!("{}", node))).unwrap();
}







////////////////////////////////////////////////////////////////////////////////
//                      High level, safe query functions
////////////////////////////////////////////////////////////////////////////////



/**
    Returns the first node in a list of nodes. Performs 2 uwnwraps
    so it will crash if the list is empty or None
*/
pub fn first_node(list: Option<Vec<u64>>) -> Option<u64>
{
    list.unwrap().pop()
}




/**
    Returns the root node
*/
pub fn get_root_node() -> u64
{
    first_node(node_query("@/")).unwrap()
}





/**
  Returns all the nodes that currently exist
*/
pub fn get_all_nodes() -> Vec<u64>
{
    node_query_with_flag("",  "").unwrap()
}





/**
  Querys bspc to check if a node exists
*/
pub fn is_node_descendant(parent: &json::Object, child: u64) -> bool
{
    get_node_descendants(parent).contains(&child)
}





/**
  Checks wether or not a node exists
*/
pub fn get_node_exists(id: u64) -> bool
{
    get_all_nodes().contains(&id)
}




/**
    Querys bspc for the currently focused node
*/
pub fn get_focused_node() -> Option<u64>
{
    //node_query("").unwrap().pop().unwrap()
    first_node(node_query(""))
}





/**
  Returns a list of all nodes on the specified desktop 
 */
pub fn get_nodes_on_desktop(desktop: u64) -> Vec<u64>
{
    node_query_with_flag(&get_desktop_name(desktop), "-d").unwrap()
}





/**
  Returns the neighbour of a specified node
 */
pub fn get_neighbouring_node(node: u64, direction: CardinalDirection) -> Option<u64>
{
    let query = format!("{}#{}", get_node_name(node), direction.as_str());
    node_query(&query).unwrap().pop()
}





/**
    Gets the subtree of node as a JSON object
*/
pub fn get_node_json(node: u64) -> json::Object
{
    let node_str = format!("{}", node);

    let str_json = call_program("bspc", &vec!("query", "-T", "-n", &node_str)).unwrap();

    json::Json::from_str(&str_json).unwrap().as_object().unwrap().clone()
}





/**
    Returns the split type of a node.
*/
pub fn get_node_split_direction(node_json: &json::Object) -> SplitDirection
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
pub fn get_node_children(node_json: &json::Object) -> Option<(json::Object, json::Object)>
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
pub fn find_target_stack(root: &json::Object, direction: &SplitDirection) -> Vec<u64>
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
pub fn get_node_id(node_json: &json::Object) -> u64
{
    node_json.get("id").unwrap().as_u64().unwrap()
}





/**
  Returns the a string representation of the ID of a node that can be interpreted by bspc
  (0x...)
*/
pub fn get_node_name(id: u64) -> String
{
    format!("0x{:X}", id)
}



/**
  Returns the a string representation of the ID of a desktop that can be interpreted by bspc
  (0x...)
*/
pub fn get_desktop_name(id: u64) -> String
{
    format!("0x{:X}", id)
}




/**
  Returns the list of directions you have to take from a node
  to a descendant. The list is in a reverse order so in order to walk the
  path to the bottom node, you should pop the resulting vector
*/
pub fn find_path_to_node(root: &json::Object, target: u64) -> Option<Vec<Children>>
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
                //path.insert(0, Children::First);
                path.push(Children::First);
                return Some(path);
            },
            (_, Some(mut path)) => {
                //path.insert(0, Children::Second);
                path.push(Children::Second);
                return Some(path);
            }
        }
    }
}



/**
 Returns all the descendant nodes of a specified node
*/
pub fn get_node_descendants(root: &json::Object) -> Vec<u64>
{
    fn tail_recursion_helper(root: &json::Object, buffer: &mut Vec<u64>)
    {
        buffer.push(get_node_id(root));
        match get_node_children(root)
        {
            None => {},
            Some(children) =>
            {
                let (first, second) = children;

                tail_recursion_helper(&first, buffer);
                tail_recursion_helper(&second, buffer);
            }
        }
    }

    let mut result = vec!();
    
    tail_recursion_helper(root, &mut result);

    result
}





/**
    Counts the amount of descendants that a node has
*/
pub fn count_node_descendant_leaves(root: &json::Object) -> u64
{
    get_node_descendant_leaves(root).len() as u64
}





/**
  Returns all the leaf nodes that are descendants of root
*/
pub fn get_node_descendant_leaves(root: &json::Object) -> Vec<u64>
{
    let children = get_node_children(root);

    match children
    {
        None => vec!(get_node_id(root)),
        Some((first, second)) =>
        {
            let mut second_result = get_node_descendant_leaves(&second);
            let mut first_result = get_node_descendant_leaves(&first);
            first_result.append(&mut second_result);

            first_result
        }
    }
}




pub fn focus_node_by_path(
        node_json: &json::Object,
        mut remaining_path: Vec<Children>,
        resize_directions: &(ResizeDirection, ResizeDirection)
    )
{
    let current_intersection = remaining_path.pop();

    //Find which node should be traversed and which node should be balanced
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

    //Calculate the ratio that we need to change the current node to
    let balance_node_size = 0.05 * count_node_descendant_leaves(&balance_node) as f32;

    let ratio = match should_balance_first
    {
        true => balance_node_size,
        false => 1. - balance_node_size
    };

    //Get the names of the nodes we want to change
    let current_node_name = get_node_name(get_node_id(node_json));

    //Apply the transformations
    node_change_ratio(&current_node_name, ratio);
    node_balance(get_node_id(&balance_node));

    //Dig deeper
    focus_node_by_path(&traverse_node, remaining_path, resize_directions);
}




////////////////////////////////////////////////////////////////////////////////
//                          Unit tests
////////////////////////////////////////////////////////////////////////////////
#[cfg(test)]
mod tests
{
    use super::{
        node_query
        , get_node_children
        , get_node_id
        , find_target_stack
        , SplitDirection
        , find_path_to_node
        , Children
        , count_node_descendant_leaves
        , get_node_descendants
        , is_node_descendant
    };

    use std::io::prelude::*;
    use std::fs::File;

    use rustc_serialize::json;


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
                Some(vec!(Children::First, Children::Second, Children::Second))
            );
        assert_eq!(
                find_path_to_node(&data, 0),
                None
            );

        assert_eq!(count_node_descendant_leaves(&data), 6);

        assert_eq!(get_node_descendants(&data), 
                   vec!(
                       4194621, 
                       29475921, 
                       4194628, 
                       29538275, 
                       4194636, 
                       29526298,
                       4194640,
                       4194638,
                       29541313,
                       29541339,
                       29541363,
                    )
                );

        //Check if the first child of the root is a descendant of the root
        assert_eq!(is_node_descendant(&data, 29475921), true);
        //Check if the child is a descendant of its parent
        assert_eq!(is_node_descendant(&get_node_children(&data).unwrap().0, 4194621), false);
    }
}
