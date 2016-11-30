
use std::process::Command;

use std::string::String;
use std::vec::Vec;

/**
    Calls a system program with the specified arguments as a vector

    Returns the output as UTF8 if successfull, Err(String) if not
 */
pub fn call_program(program_name: &str, args: &Vec<&str>) -> Result<String, String>
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


#[cfg(test)]
mod CallProgramTests
{
    use super::*;

    #[test]
    fn basic_call_test()
    {
        assert_eq!(call_program("sh", &vec!("-c", "echo hello")).unwrap(), "hello\n".to_string());

        assert!(call_program("yoloswagmannen", &vec!("-c", "echo hello")).is_err());
    }
}
