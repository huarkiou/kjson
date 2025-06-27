pub struct Stack<T> {
    stack: Vec<T>,
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Self { stack: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn push_bytes(&mut self, bytes: &[T])
    where
        T: Clone,
    {
        for b in bytes {
            self.stack.push(b.clone());
        }
    }

    pub fn push_byte(&mut self, b: T) {
        self.stack.push(b);
    }

    pub fn pop_bytes(&mut self, size: usize) -> Vec<T> {
        let len = self.stack.len();
        if size <= len {
            let removed: Vec<T> = self.stack.drain(len - size..).collect();
            removed
        } else {
            panic!("Not enough elements in VecDeque");
        }
    }
}
