use alloc::vec::Vec;

#[derive(Debug)]
pub struct UserBuffer {
    pub buffers: Vec<&'static mut [u8]>,
}

impl UserBuffer {
    pub fn new(buffers: Vec<&'static mut [u8]>) -> Self {
        Self { buffers }
    }

    pub fn len(&self) -> usize {
        self.buffers.iter().fold(0, |acc, x| acc + x.len())
    }
}
