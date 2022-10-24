#[derive(Debug, Clone)]
pub(crate) enum Command {
    Flush,
    Write(Vec<u8>),
}
