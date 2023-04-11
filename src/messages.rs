#[derive(Clone, Debug)]
pub enum Message {
    IncrementEntry(usize),
    UrlChanged(String),
    NameChanged(String),
    AddEntryBuilder,
    RemoveEntry(usize),
    Fetch(usize),
    CopyLink(usize),
}
impl Message {
    pub fn needs_saving(&self) -> bool {
        use Message::*;
        match self {
            IncrementEntry(_) => true,
            UrlChanged(_) => false,
            NameChanged(_) => false,
            AddEntryBuilder => true,
            RemoveEntry(_) => true,
            Fetch(_) => false,
            CopyLink(_) => false,
        }
    }
}
