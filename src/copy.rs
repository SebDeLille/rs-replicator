use std::fmt::{Display, Formatter};
use std::fs;

#[derive(PartialEq)]
pub enum ChangeType {NEW, CHANGE, DELETE, STOP}

pub struct FileChange {
    pub kind: ChangeType,
    pub path: String,
    pub destination: String
}

impl Display for ChangeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ChangeType::NEW => write!(f, "{}", "NEW"),
            ChangeType::CHANGE => write!(f, "{}", "CHANGE"),
            ChangeType::DELETE => write!(f, "{}", "DELETE"),
            ChangeType::STOP => write!(f, "{}", "STOP"),
        }
    }
}

pub fn manage_change(change: &FileChange) {
    println!("kind: {}, file:{}",change.kind, change.path);
    if change.kind == ChangeType::CHANGE {
        if change.path.ends_with(".py") {
            let p = change.destination.clone() + "/test.txt";
            fs::copy(&change.path, &p);
        }
    }
}