
use bspwm;

#[derive(Debug, RustcEncodable, RustcDecodable)]
pub enum Command
{
    CreateStack,
    RemoveFocused,
    IsFocusedInStack,
    Move(bspwm::CardinalDirection),
    FocusCurrent
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

