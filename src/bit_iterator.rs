pub struct BitIterator<I: Iterator<Item = u8>> {
    pub iter: I,
    pub bits: u8, // > 0
    pub curr_item: u8,
    pub curr_bit: u8,
}

impl<I: Iterator<Item = u8>> Iterator for BitIterator<I> {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_bit == self.bits {
            self.curr_bit = 0;
            if let Some(new_item) = self.iter.next() {
                self.curr_item = new_item;
            } else {
                return None;
            }
        }
        let result = Some(self.curr_item & (1 << self.curr_bit) != 0);
        self.curr_bit += 1;
        result
    }
}
