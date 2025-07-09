#[derive(Debug, Clone)]
pub enum GuiOutMessage {
    SaveHotKeys(String),
}

#[derive(Debug, Clone)]
pub enum GuiInMessage {
    Exit,
    Show,
}
