#[derive(Debug)]
pub enum Topic {
    Block,
    Tx,
}

impl From<&str> for Topic {
    fn from(topic: &str) -> Self {
        if topic == "tx" {
            Self::Tx
        } else {
            Self::Block
        }
    }
}

impl From<Topic> for String {
    fn from(topic: Topic) -> Self {
        match topic {
            Topic::Block => "block".into(),
            Topic::Tx => "tx".into(),
        }
    }
}
