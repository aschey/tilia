#[derive(Debug)]
pub(crate) enum Command {
    Flush,
    Write(Vec<u8>),
}
