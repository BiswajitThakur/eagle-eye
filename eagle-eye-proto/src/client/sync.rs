use std::{
    fmt,
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream},
    path::PathBuf,
};

use ee_stream::EStreamSync;

use crate::{
    FlowControl,
    task::{ExecuteResult, TaskSync},
    utils::handle_auth_on_client_sync,
};

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
    pub fn send<U: io::Write, T: for<'a> TaskSync<&'a mut Self, U>>(
        &mut self,
        task: T,
        http: U,
    ) -> io::Result<ExecuteResult> {
        let mut buf = [0; 1];
        write!(self, "{}\n", T::id())?;
        self.flush()?;
        self.read_exact(&mut buf)?;
        let flow = FlowControl::try_from(buf);
        if flow.is_err() {
            return Err(io::Error::other("Invalid Flow"));
        }
        match flow.unwrap() {
            FlowControl::Close | FlowControl::StopServer => Ok(ExecuteResult::UnknownTask),
            FlowControl::Continue => task.execute_on_client(self, http),
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

pub struct ClientSync {
    id: u128,
    // devices: Vec<Device>,
    log: Option<PathBuf>,
}

impl Default for ClientSync {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientSync {
    pub fn new() -> Self {
        Self {
            id: 0,
            // devices: Vec::new(),
            log: None,
        }
    }
    /*
    pub fn get_device_by_id(&self, id: u128) -> Option<&Device> {
        self.devices.iter().find(|&v| v.id == id)
    }
    pub fn push_device(&mut self, device: Device) {
        self.devices.push(device);
    }*/
    pub fn log<T: Into<PathBuf>>(mut self, path: T) -> Self {
        self.log = Some(path.into());
        self
    }
    pub fn connect<const N: usize>(
        &self,
        key: [u8; 32],
        stream: TcpStream,
    ) -> io::Result<TaskSenderSync<N, TcpStream, TcpStream>> {
        let stream1 = stream;
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
