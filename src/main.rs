use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::ops::DerefMut;
use std::rc::Rc;

mod atom;



fn main() {
    // Read file character by character
    let mut file = File::open("example.sink").unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let mut state: Box<dyn atom::Atom> =  Box::new(atom::Group::new(atom::GroupType::Root, None));
    for c in contents.chars() {
         state = state.read_char(c)
    }
    println!("{}",state.debug_str(1));
}