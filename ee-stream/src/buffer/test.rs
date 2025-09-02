use std::io::{self, Cursor, Read, Write};

use crate::buffer::BufReadWriter;

struct TestR<T> {
    reader: Cursor<T>,
}

impl<T: AsRef<[u8]>> TestR<T> {
    fn new(v: T) -> Self {
        Self {
            reader: Cursor::new(v),
        }
    }
}

impl<T> io::Write for TestR<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
    fn write_all(&mut self, _buf: &[u8]) -> io::Result<()> {
        Ok(())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<T: AsRef<[u8]>> io::Read for TestR<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

#[test]
fn test_buf_read_writer_read_1() {
    let mut buf = [0; 2];
    let mut reader = BufReadWriter::with_read_capacity(8, TestR::new("hello world.a.b.z"));
    assert_eq!(reader.read_buffer(), &[]);

    assert_eq!(reader.read(&mut buf[0..1]).unwrap(), 1);
    assert_eq!(buf[0], b'h');
    assert_eq!(reader.read_buffer(), b"ello wo");

    assert_eq!(reader.read(&mut buf).unwrap(), 2);
    assert_eq!(&buf, b"el");
    assert_eq!(reader.read_buffer(), b"lo wo");

    assert_eq!(reader.read(&mut buf).unwrap(), 2);
    assert_eq!(&buf, b"lo");
    assert_eq!(reader.read_buffer(), b" wo");

    assert_eq!(reader.read(&mut buf).unwrap(), 2);
    assert_eq!(&buf, b" w");
    assert_eq!(reader.read_buffer(), b"o");

    assert_eq!(reader.read(&mut buf).unwrap(), 1);
    assert_eq!(&buf[0..1], &*b"o");
    assert_eq!(reader.read_buffer(), b"");

    assert_eq!(reader.read(&mut buf).unwrap(), 2);
    assert_eq!(&buf, b"rl");
    assert_eq!(reader.read_buffer(), b"d.a.b.");

    assert_eq!(reader.read(&mut buf).unwrap(), 2);
    assert_eq!(&buf, b"d.");
    assert_eq!(reader.read_buffer(), b"a.b.");

    assert_eq!(reader.read(&mut buf).unwrap(), 2);
    assert_eq!(&buf, b"a.");
    assert_eq!(reader.read_buffer(), b"b.");

    assert_eq!(reader.read(&mut buf).unwrap(), 2);
    assert_eq!(&buf, b"b.");
    assert_eq!(reader.read_buffer(), b"");

    assert_eq!(reader.read(&mut buf).unwrap(), 1);
    assert_eq!(&buf[0..1], &*b"z");
    assert_eq!(reader.read_buffer(), b"");

    assert_eq!(reader.read(&mut buf).unwrap(), 0);
    assert_eq!(reader.read_buffer(), b"");

    assert_eq!(reader.read(&mut buf).unwrap(), 0);
    assert_eq!(reader.read_buffer(), b"");
}

#[test]
fn test_buf_read_writer_read_2() {
    let mut buf = [0; 9];
    let mut reader = BufReadWriter::with_read_capacity(8, TestR::new("hello world"));
    assert_eq!(reader.read_buffer(), &[]);

    assert_eq!(reader.read(&mut buf).unwrap(), 9);
    assert_eq!(&buf, b"hello wor");
    assert_eq!(reader.read_buffer(), &[]);

    assert_eq!(reader.read(&mut buf).unwrap(), 2);
    assert_eq!(&buf[0..2], b"ld");
    assert_eq!(reader.read_buffer(), &[]);

    assert_eq!(reader.read(&mut buf).unwrap(), 0);
    assert_eq!(reader.read_buffer(), &[]);
}

#[test]
fn test_buf_read_writer_read_3() {
    let mut buf = [0; 9];
    let mut reader = BufReadWriter::with_read_capacity(8, TestR::new("hello world"));

    assert_eq!(reader.read(&mut buf[0..1]).unwrap(), 1);
    assert_eq!(buf[0], b'h');
    assert_eq!(reader.read_buffer(), b"ello wo");

    assert_eq!(reader.read(&mut buf).unwrap(), 7);
    assert_eq!(&buf[0..7], b"ello wo");
    assert_eq!(reader.read_buffer(), &[]);

    assert_eq!(reader.read(&mut buf).unwrap(), 3);
    assert_eq!(&buf[0..3], b"rld");
    assert_eq!(reader.read_buffer(), &[]);

    assert_eq!(reader.read(&mut buf).unwrap(), 0);
    assert_eq!(reader.read_buffer(), &[]);
}

#[test]
fn test_buf_read_writer_write_1() {
    let mut out = Vec::<u8>::new();
    let mut writer = BufReadWriter::with_write_capacity(8, &mut out);

    assert_eq!(writer.inner_ref().as_slice(), &[]);

    writer.write_all(b"hello").unwrap();
    assert_eq!(writer.write_buffer(), b"hello");
    assert_eq!(writer.inner_ref().as_slice(), &[]);
    writer.flush().unwrap();
    assert_eq!(writer.inner_ref().as_slice(), b"hello");
    assert_eq!(writer.write_buffer(), &[]);

    writer.write_all(b"1234567890").unwrap();
    assert_eq!(writer.write_buffer(), &[]);
    assert_eq!(writer.inner_ref().as_slice(), b"hello1234567890");

    writer.write_all(b"world").unwrap();
    assert_eq!(writer.write_buffer(), b"world");
    assert_eq!(writer.inner_ref().as_slice(), b"hello1234567890");

    writer.write_all(b"abc").unwrap();
    assert_eq!(writer.write_buffer(), b"worldabc");
    assert_eq!(writer.inner_ref().as_slice(), b"hello1234567890");

    writer.write_all(b"ABC").unwrap();
    assert_eq!(writer.write_buffer(), b"ABC");
    assert_eq!(writer.inner_ref().as_slice(), b"hello1234567890worldabc");

    writer.flush().unwrap();
    assert_eq!(writer.write_buffer(), &[]);
    assert_eq!(writer.inner_ref().as_slice(), b"hello1234567890worldabcABC");

    drop(writer);

    assert_eq!(&out, b"hello1234567890worldabcABC");
}

#[test]
fn test_buf_read_writer_write_2() {
    let mut out = Vec::<u8>::new();
    let mut writer = BufReadWriter::with_write_capacity(8, &mut out);

    writer.write_all(b"hello").unwrap();
    assert_eq!(writer.write_buffer(), b"hello");
    assert_eq!(writer.inner_ref().as_slice(), &[]);
    drop(writer);
    assert_eq!(&out, b"hello");
}
