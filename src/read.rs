use std::io::{self, Read};

/// An extension for [std::io::Read] that does the allocations for you
pub trait ReadExt: Read {
    /// Read a fixed number of bytes specified at compile time into a fixed-size array
    fn read_const_num_of_bytes<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        let mut buf = [0; N];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Read a little endian u32
    fn read_u32_le(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.read_const_num_of_bytes()?))
    }
}

impl<T: Read> ReadExt for T {}
