use std::cmp;
use std::io::{self, BufRead, Read, Write};

#[derive(Debug)]
enum Phase {
    Read { head: usize, tail: usize },
    Write {
        read_tail: usize,
        head: usize,
        tail: usize,
    },
}
impl Phase {
    fn head(&self) -> usize {
        match *self {
            Phase::Read { head, .. } => head,
            Phase::Write { head, .. } => head,
        }
    }
    fn tail(&self) -> usize {
        match *self {
            Phase::Read { tail, .. } => tail,
            Phase::Write { tail, .. } => tail,
        }
    }
    fn head_mut(&mut self) -> &mut usize {
        match *self {
            Phase::Read { ref mut head, .. } => head,
            Phase::Write { ref mut head, .. } => head,
        }
    }
    fn tail_mut(&mut self) -> &mut usize {
        match *self {
            Phase::Read { ref mut tail, .. } => tail,
            Phase::Write { ref mut tail, .. } => tail,
        }
    }
}

#[derive(Debug)]
pub struct Buffer {
    bytes: Vec<u8>,
    phase: Phase,
    max_len: usize,
}
impl Buffer {
    pub fn new(min_len: usize, max_len: usize) -> Self {
        assert!(min_len <= max_len);
        Buffer {
            bytes: vec![0; min_len],
            phase: Phase::Read { head: 0, tail: 0 },
            max_len: max_len,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.phase.head() == self.phase.tail()
    }
    pub fn enter_read_phase(&mut self) {
        if let Phase::Write { read_tail, .. } = self.phase {
            self.phase = Phase::Read {
                head: 0,
                tail: read_tail,
            };
        }
    }
    pub fn enter_write_phase(&mut self) {
        if let Phase::Read { head, tail } = self.phase {
            let read_tail = tail - head;
            self.bytes.drain(..read_tail);
            self.phase = Phase::Write {
                read_tail: read_tail,
                head: read_tail,
                tail: read_tail,
            };
        }
    }
    pub fn fill_from<R: Read>(&mut self, reader: &mut R) -> io::Result<usize> {
        self.expand_if_needed();
        self.check_overflow()?;
        let tail = self.phase.tail_mut();
        let buf = &mut self.bytes[*tail..];
        let read_size = reader.read(buf)?;
        *tail += read_size;
        Ok(read_size)
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[self.phase.head()..self.phase.tail()]
    }
    fn expand_if_needed(&mut self) {
        if self.phase.tail() == self.bytes.len() {
            let new_len = cmp::min(self.bytes.len() * 2, self.max_len);
            self.bytes.resize(new_len, 0);
        }
    }
    fn check_overflow(&self) -> io::Result<()> {
        if self.phase.tail() == self.max_len {
            let message = format!("Buffer for HTTP non-body part is overflowed: max_len={}",
                                  self.max_len);
            Err(io::Error::new(io::ErrorKind::WriteZero, message))
        } else {
            Ok(())
        }
    }
}
impl Read for Buffer {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let size;
        {
            let src = self.as_slice();
            size = cmp::min(buf.len(), src.len());
            (&mut buf[..size]).copy_from_slice(&src[..size]);
        }
        self.consume(size);
        Ok(size)
    }
}
impl BufRead for Buffer {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(self.as_slice())
    }
    fn consume(&mut self, amt: usize) {
        let tail = self.phase.tail();
        let head = self.phase.head_mut();
        *head += amt;
        assert!(*head <= tail)
    }
}
impl Write for Buffer {
    fn write(&mut self, mut buf: &[u8]) -> io::Result<usize> {
        self.fill_from(&mut buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.check_overflow()
    }
}
