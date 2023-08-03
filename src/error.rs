#[derive(Debug)]
pub struct ReplicatorError {
    msg: String,
}

impl ReplicatorError {
    pub fn new(msg: String) -> Self {
        ReplicatorError {
            msg,
        }
    }

    pub fn msg(&self) -> String {
        self.msg.clone()
    }
}