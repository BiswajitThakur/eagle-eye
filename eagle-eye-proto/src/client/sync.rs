use std::{
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
    path::PathBuf,
};

use crate::{
    FlowControl,
    stream::EagleEyeStreamSync,
    task::{ExecuteResult, TaskSync},
    utils::handle_auth_on_client_sync,
};

pub struct TaskSenderSync<const N: usize, R: io::Read, W: io::Write> {
    stream: EagleEyeStreamSync<N, R, W>,
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
    pub fn new(value: EagleEyeStreamSync<N, R, W>) -> Self {
        Self { stream: value }
    }
}

impl<const N: usize, R: io::Read, W: io::Write> TaskSenderSync<N, R, W> {
    pub fn send<U: io::Write, E: io::Write, T: for<'a> TaskSync<&'a mut Self, U, E>>(
        &mut self,
        task: T,
        ok: U,
        err: E,
    ) -> io::Result<ExecuteResult> {
        let mut buf = [0; 1];
        writeln!(self, "{}", <T as TaskSync<&mut Self, U, E>>::id())?;
        self.flush()?;
        self.read_exact(&mut buf)?;
        let flow = FlowControl::try_from(buf);
        if flow.is_err() {
            return Err(io::Error::other("Invalid Flow"));
        }
        match flow.unwrap() {
            FlowControl::Close | FlowControl::StopServer => {
                self.read_exact(&mut buf)?;
                let exe = ExecuteResult::try_from(buf);
                if exe.is_err() {
                    return Err(io::Error::other("Invalid Execution Result"));
                }
                Ok(exe.unwrap())
            }
            FlowControl::Continue => task.execute_on_sender(self, ok, err),
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

pub struct ClientSync<const N: usize> {
    log: Option<PathBuf>,
}

impl<const N: usize> ClientSync<N> {
    pub fn new() -> Self {
        Self { log: None }
    }
    pub fn log<T: Into<PathBuf>>(mut self, path: T) -> Self {
        self.log = Some(path.into());
        self
    }
    pub fn connect(
        &self,
        key: [u8; 32],
        addr: SocketAddr,
    ) -> io::Result<TaskSenderSync<N, TcpStream, TcpStream>> {
        let stream1 = TcpStream::connect(addr)?;
        let stream2 = stream1.try_clone()?;
        let e_stream = match handle_auth_on_client_sync::<N, _, _>(key, stream1, stream2)? {
            Some(v) => v,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Wrong Password",
                ));
            }
        };
        Ok(TaskSenderSync::new(e_stream))
    }
}
