use std::io;

use crate::FlowControl;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum ExecuteResult {
    Ok = 0,
    InvalidRequest = 1,
    UnknownTask = 2,
}
/*
impl TryFrom<u8> for ExecuteResult {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Ok),
            1 => Ok(Self::InvalidRequest),
            2 => Ok(Self::UnknownTask),
            _ => Err(()),
        }
    }
}
*/
impl TryFrom<[u8; 1]> for ExecuteResult {
    type Error = ();
    fn try_from(value: [u8; 1]) -> Result<Self, Self::Error> {
        match value {
            [0] => Ok(Self::Ok),
            [1] => Ok(Self::InvalidRequest),
            [2] => Ok(Self::UnknownTask),
            _ => Err(()),
        }
    }
}

impl ExecuteResult {
    pub fn to_be_bytes(&self) -> [u8; 1] {
        let v = *self as u8;
        v.to_be_bytes()
    }
}

pub trait TaskSync<T: io::Read + io::Write, W: io::Write, E: io::Write> {
    fn id() -> &'static str;
    fn execute(&self, stream: T, ok: W, err: E) -> io::Result<ExecuteResult>;
    fn execute_on_listener(mut stream: T) -> io::Result<FlowControl> {
        let mut buf = [0; 1];
        stream.read_exact(&mut buf)?;
        let flow = FlowControl::try_from(buf);
        if flow.is_err() {
            stream.write_all(&FlowControl::Close.to_be_bytes())?;
            stream.write_all(&ExecuteResult::InvalidRequest.to_be_bytes())?;
            return Ok(FlowControl::Close);
        }
        // consuming the task id
        loop {
            let n = stream.read(&mut buf)?;
            if n == 0 || buf[0] == b'\n' {
                break;
            }
        }
        stream.write_all(&FlowControl::Close.to_be_bytes())?;
        stream.write_all(&ExecuteResult::UnknownTask.to_be_bytes())?;
        Ok(flow.unwrap_or(FlowControl::Close))
    }
}

pub struct TaskRegisterySync<T> {
    _default: fn(T) -> io::Result<(FlowControl, T)>,
    tasks: Vec<(&'static str, fn(T) -> io::Result<(FlowControl, T)>)>,
}

impl<T> TaskRegisterySync<T> {
    pub fn new(default: fn(T) -> io::Result<(FlowControl, T)>) -> Self {
        Self {
            _default: default,
            tasks: Vec::new(),
        }
    }
    pub fn register(&mut self, id: &'static str, f: fn(T) -> io::Result<(FlowControl, T)>) {
        for &(v, _) in self.tasks.iter() {
            assert_ne!(id, v, "already exists with this id...");
        }
        self.tasks.push((id, f));
        self.tasks.sort_by(|a, b| a.0.cmp(b.0));
    }
    pub fn get<S: AsRef<str>>(&self, id: S) -> fn(T) -> io::Result<(FlowControl, T)> {
        let v = id.as_ref();
        match self.tasks.binary_search_by_key(&v, |&(a, _)| a) {
            Ok(index) => self.tasks.get(index).map(|m| m.1).unwrap_or(self._default),
            Err(_) => self._default,
        }
    }
}
