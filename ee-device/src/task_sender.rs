use std::io::{self, Read, Write};

use ee_http::HttpRequest;
use ee_stream::{EStreamSync, FlowControl};
use ee_task::{ExeSenderSync, ExecuteResult};

pub struct TaskSenderSync<const N: usize, R: io::Read, W: io::Write> {
    stream: EStreamSync<N, R, W>,
}

impl<const N: usize, R: io::Read, W: io::Write> io::Read for TaskSenderSync<N, R, W> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

impl<const N: usize, R: io::Read, W: io::Write> io::Write for TaskSenderSync<N, R, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.write(buf)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

impl<const N: usize, R: io::Read, W: io::Write> TaskSenderSync<N, R, W> {
    pub fn new(value: EStreamSync<N, R, W>) -> Self {
        Self { stream: value }
    }
}

impl<const N: usize, R: io::Read, W: io::Write> TaskSenderSync<N, R, W> {
    pub fn send<U: io::Write, T: for<'a> ExeSenderSync<&'a mut Self, U>>(
        &mut self,
        task: T,
        req: &mut HttpRequest,
        http: U,
    ) -> io::Result<ExecuteResult> {
        let mut buf = [0; 1];
        writeln!(self, "{}", T::id())?;
        self.flush()?;
        self.read_exact(&mut buf)?;
        let flow = FlowControl::try_from(buf);
        if flow.is_err() {
            return Err(io::Error::other("Invalid Flow"));
        }
        match flow.unwrap() {
            FlowControl::Close | FlowControl::StopServer => Ok(ExecuteResult::UnknownTask),
            FlowControl::Continue => task.execute_on_sender(self, req, http),
        }
    }

    pub fn end(&mut self) -> io::Result<()> {
        self.write_all(b":end:\n")?;
        self.flush()
    }

    pub fn stop_server(&mut self) -> io::Result<()> {
        self.write_all(b":stop-server:\n")?;
        self.flush()
    }
}
