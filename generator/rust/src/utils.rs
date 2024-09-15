// utils.rs

use heapless::Vec;
use minicbor::encode::Write;

/// An error indicating the end of an array.
#[derive(Debug)]
pub struct EndOfArray(());

impl core::fmt::Display for EndOfArray {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.write_str("end of array")
    }
}

pub struct Buffer<const N: usize> {
    inner: Vec<u8, N>,
}

impl<const N: usize> Buffer<N> {
    pub fn new() -> Self {
        Self {
            inner: Vec::new(),
        }
    }

    pub fn extend_from_slice(&mut self, other: &[u8]) -> Result<(), ()> {
        self.inner.extend_from_slice(other)
    }

    pub fn as_slice(&self) -> &[u8] {
        self.inner.as_slice()
    }

    pub fn len(&self) -> usize {
      self.inner.len()
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }
}

impl<const N: usize> Write for Buffer<N> {
    type Error = EndOfArray;

    fn write_all(&mut self, buf: &[u8]) -> Result<(), Self::Error> {
        if self.inner.capacity() - self.inner.len() < buf.len() {
            return Err(EndOfArray(()));
        }

        let _ = self.inner.extend_from_slice(buf);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_write() {
        let mut buf = Buffer::<32>::new();

        minicbor::encode(&42, &mut buf).unwrap();
        assert_eq!(buf.as_slice(), &[0x18, 0x2A]);

        minicbor::encode(&(1, 2, 3), &mut buf).unwrap();
        assert_eq!(buf.as_slice(), &[0x18, 0x2A, 0x83, 0x01, 0x02, 0x03]);

        buf.clear();
        assert_eq!(buf.as_slice(), &[]);

        let mut buf2 = Buffer::<2>::new();
        let result = minicbor::encode(&(1, 2, 3), &mut buf2);
        assert!(result.is_err());
    }
}
