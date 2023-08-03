use std::fmt::{Display, Formatter};
use std::fs;

#[derive(PartialEq, Clone)]
pub enum ChangeType {NEW, CHANGE, DELETE, STOP}

pub struct FileChange {
    pub kind: ChangeType,
    pub path: String,
    pub destination: String,
    pub exceptions: Vec<String>
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

impl Display for FileChange {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{} - {} - {} - {:?}", self.kind, self.path, self.destination, self.exceptions)
    }
}
fn is_valid(change: &FileChange) -> bool {
    for ext in change.exceptions.clone() {
        if change.path.ends_with(ext.as_str()) {
            return false
        }
    }
    true
}

pub fn manage_change(change: &FileChange) {
    println!("{}", change);
    match change.kind {
        ChangeType::CHANGE => {
            if is_valid(change) {
                let p = change.destination.clone() + "/test.txt";
                if let Err(e) = fs::copy(&change.path, &p) {
                    println!("{}", e);
                }
            }
        },
        _ => ()
    }
}