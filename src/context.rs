use crate::stack::Stack;

pub struct Context<'a> {
    pub bytes: &'a [u8],
    pub stack: Stack<u8>,
}

impl<'a> Context<'a> {
    pub fn new(json: &'a [u8]) -> Self {
        Self {
            bytes: json,
            stack: Stack::<u8>::new(),
        }
    }

    pub fn step(&mut self) -> Option<u8> {
        let &b = self.bytes.first()?;
        self.bytes = &self.bytes[1..];
        Some(b)
    }
}
