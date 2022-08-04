use std::cell::RefCell;
use std::rc::Rc;

pub trait Atom {
    fn read_char(self: Box<Self>, c: char) -> Box<dyn Atom>;
    fn name(self) -> String;
    fn debug_str(&self, tabs: usize) -> String;
}
const OPERATOR_SYMBOL: &str = "#@!%^&*-_+><=/\\~`,.|?:";
const FINAL_OPERATOR: &str = ";";
fn take_parent_and_add_child(parent: Box<Group>, child: Box<dyn Atom>) -> Box<Group>{
    let v = parent.as_ref().children.clone();
    (*v.clone()).borrow_mut().push(child);
    return parent;
}

// Group atom //==============================================================
pub enum GroupType {
    Root,
    Curly,
    Square,
    Curved
}

pub struct Group {
    parent: Option<Box<Group>>,
    children: Rc<RefCell<Vec<Box<dyn Atom>>>>,
    group_type: GroupType
}

impl Group {
    pub fn new(group_type: GroupType, parent: Option<Box<Group>>) -> Group {
        Group {
            parent,
            children: Rc::new(RefCell::new(Vec::new())),
            group_type
        }
    }


    pub fn read_default(mut self: Box<Self>, c: char) -> Box<dyn Atom> {
        match c {
            '{' => return Box::new(Group::new(GroupType::Curly, Some(self))),
            '(' => return Box::new(Group::new(GroupType::Curved, Some(self))),
            '[' => return Box::new(Group::new(GroupType::Square, Some(self))),
            '\"' => return Box::new(StringLiteral::new(self)),
            _ => {
                if c.is_ascii_alphabetic() {
                    return Box::new(Identifier::new(self)).read_char(c);
                }else if c.is_ascii_digit(){
                    return Box::new(IntegerLiteral::new(self)).read_char(c);
                }else if OPERATOR_SYMBOL.contains(c) || FINAL_OPERATOR.contains(c){
                    return Box::new(Operator::new(self)).read_char(c);
                }
                return self;
            }
        };
    }
}
impl Atom for Group {
    fn read_char(mut self: Box<Self>, c: char) -> Box<dyn Atom> {
        match self.group_type {
            GroupType::Root => {
                self.read_default(c)
            },
            GroupType::Curly => {
                match c {
                    '}' => take_parent_and_add_child(self.parent.take().unwrap(), self),
                    _ => self.read_default(c)
                }
            },
            GroupType::Square => {
                match c {
                    ']' => take_parent_and_add_child(self.parent.take().unwrap(), self),
                    _ => self.read_default(c)
                }
            },
            GroupType::Curved => {
                match c {
                    ')' => take_parent_and_add_child(self.parent.take().unwrap(), self),
                    _ => self.read_default(c)
                }
            }
        }
    }

    fn name(self) -> String {
         match self.group_type {
             GroupType::Root => "Group Root",
             GroupType::Curly => "Group Curly",
             GroupType::Square => "Group Square",
             GroupType::Curved => "Group Curved"
         }.to_owned()
    }

    fn debug_str(&self, tabs: usize) -> String {
        let mut build_string = String::new();

        build_string.push_str(match self.group_type {
            GroupType::Root => "Root{\n",
            GroupType::Curly => " curly{\n",
            GroupType::Square => " square[\n",
            GroupType::Curved => " curved("
        });
        build_string.push_str(&*match self.group_type {
            GroupType::Root => "\t".repeat(tabs),
            GroupType::Curly => "\t".repeat(tabs),
            GroupType::Square => "\t".repeat(tabs),
            GroupType::Curved => "".to_owned()
        });

        for child in self.children.borrow().iter() {
            build_string.push_str(&*child.debug_str(tabs + 1));
        }
        build_string.push_str(match self.group_type {
            GroupType::Root => "\n",
            GroupType::Curly => "\n",
            GroupType::Square => "\n",
            GroupType::Curved => ""
        });
        build_string.push_str(&*match self.group_type {
            GroupType::Root => "\t".repeat(tabs-1),
            GroupType::Curly => "\t".repeat(tabs-1),
            GroupType::Square => "\t".repeat(tabs-1),
            GroupType::Curved => "".to_owned()
        });
        build_string.push_str(match self.group_type {
            GroupType::Root => "}",
            GroupType::Curly => "}",
            GroupType::Square => "]",
            GroupType::Curved => ")"
        });
        build_string
    }
}

// Identifier atom //=========================================================
pub struct Identifier {
    parent: Option<Box<Group>>,
    value: String
}
impl Identifier {
    pub fn new(parent: Box<Group>) -> Identifier {
        Identifier {
            parent: Some(parent),
            value: String::new()
        }
    }
}
impl Atom for Identifier {
    fn read_char(mut self: Box<Self>, c: char) -> Box<dyn Atom> {
        if c.is_ascii_alphanumeric() {
            self.value.push(c);
            return self;
        }else{
            return take_parent_and_add_child(self.parent.take().unwrap(), self).read_char(c);
        }
    }

    fn name(self) -> String {
        "Identifier".to_owned()
    }

    fn debug_str(&self, tabs: usize) -> String {
        let mut build_string = String::new();
        build_string.push_str(" iden:");
        build_string.push_str(&*self.value);
        build_string
    }
}
// String literal atom //=====================================================
pub struct StringLiteral {
    parent: Option<Box<Group>>,
    value: String,
    escaped: bool
}
impl StringLiteral {
    pub fn new(parent: Box<Group>) -> StringLiteral {
        StringLiteral {
            parent: Some(parent),
            value: String::new(),
            escaped: false
        }
    }
}
impl Atom for StringLiteral {
    fn read_char(mut self: Box<Self>, c: char) -> Box<dyn Atom> {
        if self.escaped {
            self.value.push(
                match c {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '\\' => '\\',
                    '\"' => '\"',
                    '\'' => '\'',
                    _ => c
                }
            );
            self.escaped = false;
            return self;
        }
        match c {
            '\\' => {
                self.escaped = true;
                return self;
            },
            '"' => {
                return take_parent_and_add_child(self.parent.take().unwrap(), self);
            },
            _ => {
                self.value.push(c);
                return self;
            }
        }
    }

    fn name(self) -> String {
        "String Literal".to_owned()
    }

    fn debug_str(&self, tabs: usize) -> String {
        let mut build_string = String::new();
        build_string.push_str(" strLit(");
        build_string.push_str(&*self.value);
        build_string.push_str(")");
        build_string
    }
}
// Integet literal atom //=====================================================
pub struct IntegerLiteral {
    parent: Option<Box<Group>>,
    value: i64
}
impl IntegerLiteral {
    pub fn new(parent: Box<Group>) -> IntegerLiteral {
        IntegerLiteral {
            parent: Some(parent),
            value: 0
        }
    }
}
impl Atom for IntegerLiteral {
    fn read_char(mut self: Box<Self>, c: char) -> Box<dyn Atom> {
        if c.is_ascii_digit() {
            self.value = self.value * 10 + c.to_digit(10).unwrap() as i64;
            return self;
        }else if c == '.'{
            return Box::new(NumericLiteral::new(self.parent.take().unwrap(), self.value));
        }else{
            return take_parent_and_add_child(self.parent.take().unwrap(), self).read_char(c);
        }
    }

    fn name(self) -> String{
        "Integer Literal".to_owned()
    }

    fn debug_str(&self, tabs: usize) -> String {
        let mut build_string = String::new();
        build_string.push_str(" intLit(");
        build_string.push_str(&*self.value.to_string());
        build_string.push_str(")");
        build_string
    }
}
// Numeric literal atom //=====================================================
pub struct NumericLiteral {
    parent: Option<Box<Group>>,
    value: f64,
    position: i32
}
impl NumericLiteral {
    pub fn new(parent: Box<Group>, inital_value: i64) -> NumericLiteral {
        NumericLiteral {
            parent: Some(parent),
            value: inital_value as f64,
            position: 1
        }
    }
}
impl Atom for NumericLiteral {
    fn read_char(mut self: Box<Self>, c: char) -> Box<dyn Atom> {
        return if c.is_ascii_digit() {
            self.value = self.value + (c.to_digit(10).unwrap() as f64) / 10.0f64.powi(self.position);
            self.position += 1;
            return self
        } else {
            take_parent_and_add_child(self.parent.take().unwrap(), self).read_char(c)
        }
    }

    fn name(self) -> String {
        "Numeric Literal".to_owned()
    }

    fn debug_str(&self, tabs: usize) -> String {
        let mut build_string = String::new();
        build_string.push_str(" numLit(");
        build_string.push_str(&*self.value.to_string());
        build_string.push_str(")");
        build_string
    }
}
// Operator atom //===========================================================
pub struct Operator {
    parent: Option<Box<Group>>,
    value: String
}
impl Operator {
    pub fn new(parent: Box<Group>) -> Operator {
        Operator {
            parent: Some(parent),
            value: String::new()
        }
    }
}
impl Atom for Operator {
    fn read_char(mut self: Box<Self>, c: char) -> Box<dyn Atom> {
        if OPERATOR_SYMBOL.contains(c) {
            self.value.push(c);
            return self;
        }else if FINAL_OPERATOR.contains(c){
            self.value.push(c);
            return take_parent_and_add_child(self.parent.take().unwrap(), self);
        }else{
            return take_parent_and_add_child(self.parent.take().unwrap(), self).read_char(c);
        }
    }

    fn name(self) -> String {
        "Operator".to_owned()
    }

    fn debug_str(&self, tabs: usize) -> String {
        let mut build_string = String::new();
        build_string.push_str(" op(");
        build_string.push_str(&*self.value);
        build_string.push_str(")");
        build_string
    }
}
