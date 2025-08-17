use std::{
    ffi::CString,
    io::{self, BufRead, Read, Seek},
};

/// An extension for [std::io::Read] that does the allocations for you
pub trait ReadExt: Read {
    /// Read a fixed number of bytes specified at compile time into a fixed-size array
    fn read_const_num_of_bytes<const N: usize>(&mut self) -> io::Result<[u8; N]> {
        let mut buf = [0; N];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Read a specific amount of bytes
    fn read_num_of_bytes(&mut self, num: usize) -> io::Result<Vec<u8>> {
        let mut buf = vec![0; num];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Read the entire contents into a vec
    fn read_to_vec(&mut self) -> io::Result<Vec<u8>> {
        let mut buf = vec![];
        self.read_to_end(&mut buf)?;
        Ok(buf)
    }

    /// Read a little endian u32
    fn read_u32_le(&mut self) -> io::Result<u32> {
        Ok(u32::from_le_bytes(self.read_const_num_of_bytes()?))
    }

    /// Read a little endian u64
    fn read_u64_le(&mut self) -> io::Result<u64> {
        Ok(u64::from_le_bytes(self.read_const_num_of_bytes()?))
    }
}

pub fn read_utf16_name<const N: usize>(bytes: [u8; N]) -> String {
    let name: Vec<u16> = bytes
        .chunks_exact(2)
        .take_while(|b| b[0] | b[1] != 0)
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .collect();
    String::from_utf16_lossy(&name)
}

impl<T: Read> ReadExt for T {}
