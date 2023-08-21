use std::fmt::{Display, Formatter};
use std::fs;

#[derive(PartialEq, Clone)]
pub enum ChangeType { NEW, CHANGE, DELETE, STOP }

pub struct FileChange {
    pub kind: ChangeType,
    pub path: String,
    pub source: String,
    pub destination: String,
    pub exceptions: Vec<String>,
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
        write!(f, "{} - {} - {} - {} - {:?}", self.kind, self.path, self.source, self.destination, self.exceptions)
    }
}

fn is_valid(change: &FileChange) -> bool {
    for ext in change.exceptions.clone() {
        if change.path.ends_with(ext.as_str()) {
            return false;
        }
    }
    true
}

fn copy_file(change: &FileChange) {
    println!("copy");
    if is_valid(change) {
        let d = change.destination.clone() + change.path.clone().as_str();
        let s = change.source.clone() + change.path.clone().as_str();
        println!("{}", d.clone());
        println!("{}", s.clone());
        if let Err(e) = fs::copy(&s, &d) {
            println!("{}", e);
        }
    }
}

fn delete_file() {}

pub fn manage_change(change: &FileChange) {
    println!("{}", change);
    match change.kind {
        ChangeType::CHANGE => copy_file(change),
        ChangeType::NEW => copy_file(change),
        ChangeType::DELETE => delete_file(),
        _ => ()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use crate::copy::{ChangeType, FileChange, manage_change};

    fn create_dest_diretory(dir: &str) -> std::io::Result<()> {
        fs::create_dir(dir)
    }

    fn delete_dest_directory(dir: &str) {
        let r = fs::remove_dir_all(dir);
        if r.is_err() {
            println!("{} does not exist", dir);
        }
    }
    #[test]
    fn test_copy_new() {
        let dest = "./tests/resources/dest";
        delete_dest_directory(dest);
        if create_dest_diretory(dest).is_err() {
            panic!("Unable to create {} directory", dest);
        }

        let change = FileChange {
            kind: ChangeType::NEW,
            source: String::from("./tests/resources/src"),
            destination: String::from(dest),
            path: String::from("/tocopy.txt"),
            exceptions: vec![".xml".to_string()],
        };
        manage_change(&change);

        assert!(std::path::Path::new("./tests/resources/dest/tocopy.txt").exists());
    }

    #[test]
    fn test_copy_change() {
        let dest = "./tests/resources/destc";
        delete_dest_directory(dest);
        if create_dest_diretory(dest).is_err() {
            panic!("Unable to create {} directory", dest);
        }

        let change = FileChange {
            kind: ChangeType::CHANGE,
            source: String::from("./tests/resources/src"),
            destination: String::from(dest),
            path: String::from("/tocopy.txt"),
            exceptions: vec![".xml".to_string()],
        };
        manage_change(&change);

        assert!(std::path::Path::new("./tests/resources/destc/tocopy.txt").exists());

    }
}