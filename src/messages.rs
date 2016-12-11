
use bspwm;

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub enum Command
{
    CreateStack,
    IsFocusedInStack,
    Move(bspwm::Direction)
}
#[derive(Debug, RustcEncodable, RustcDecodable)]
pub enum CommandResponse
{
    Done,
    NoStackExists,
    EndOfStack,
    Yes,
    No
}

