pub mod client;
pub mod server;
pub mod stream;
pub mod task;
pub mod utils;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowControl {
    Close = 0,
    Continue = 1,
    StopServer = 2,
}

/*
impl TryFrom<u8> for FlowControl {
    type Error = ();
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Close),
            1 => Ok(Self::Continue),
            2 => Ok(Self::StopServer),
            _ => Err(()),
        }
    }
}
*/

impl TryFrom<[u8; 1]> for FlowControl {
    type Error = ();
    fn try_from(value: [u8; 1]) -> Result<Self, Self::Error> {
        match value {
            [0] => Ok(Self::Close),
            [1] => Ok(Self::Continue),
            [2] => Ok(Self::StopServer),
            _ => Err(()),
        }
    }
}

impl FlowControl {
    #[inline]
    pub fn to_be_bytes(&self) -> [u8; 1] {
        let v = *self as u8;
        v.to_be_bytes()
    }
}

/*
use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
};

use eagle_eye_proto::{
    client::ClientSync,
    server::EagleEyeServerSync,
    stream::EagleEyeStreamSync,
    task::{ExecuteResult, TaskRegisterySync, TaskSync},
};

struct GetPwd;

impl GetPwd {
    fn new() -> Self {
        Self
    }
}

impl<T: io::Read + io::Write, W: io::Write, E: io::Write> TaskSync<T, W, E> for GetPwd {
    fn id() -> &'static str {
        "pwd"
    }
    fn execute_on_sender(&self, mut stream: T, mut ok: W, _err: E) -> io::Result<ExecuteResult> {
        let mut buf = [0; 1];
        loop {
            let n = stream.read(&mut buf)?;
            if n == 0 || buf[0] == b'\n' {
                break;
            }
            ok.write_all(&buf)?;
        }
        stream.read_exact(&mut buf)?;
        let exe = ExecuteResult::try_from(buf);
        if exe.is_err() {
            return Err(io::Error::other("Invalid Execution Result"));
        }
        ok.flush()?;
        Ok(exe.unwrap())
    }
}

impl GetPwd {
    fn execute_on_receiver<T: io::Read + io::Write>(mut stream: T) -> io::Result<T> {
        stream.write_all(b"/path/home/eagle\n")?;
        stream.write_all(&ExecuteResult::Ok.to_be_bytes())?;
        stream.flush()?;
        Ok(stream)
    }
}

const KEY: [u8; 32] = [1; 32];

fn main() -> io::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 69, 69, 69)), 6969);
    let t = std::thread::spawn(move || {
        let mut registery = TaskRegisterySync::new();
        registery.register("pwd", GetPwd::execute_on_receiver);
        let listener = TcpListener::bind(addr).unwrap();
        let server: EagleEyeServerSync<EagleEyeStreamSync<1014, _, _>> =
            EagleEyeServerSync::new(KEY, registery).set_log_path("error.log");
        server
            .run(
                listener
                    .incoming()
                    .map(|v| v.map(|m| (m.try_clone().unwrap(), m))),
            )
            .unwrap();
    });
    std::thread::sleep(std::time::Duration::from_secs(1));
    let client = ClientSync::<1024>::new();
    let mut sender = client.connect(KEY, addr)?;
    let mut output = Vec::new();
    // <id>\n<flow>
    sender.send(GetPwd::new(), &mut output, io::sink())?;
    sender.send(GetPwd::new(), &mut output, io::sink())?;
    sender.send(GetPwd::new(), &mut output, io::sink())?;
    sender.send(GetPwd::new(), &mut output, io::sink())?;
    sender.send(GetPwd::new(), &mut output, io::sink())?;
    let r = sender.send(GetPwd::new(), &mut output, io::sink())?;
    let v = unsafe { String::from_utf8_unchecked(output) };
    println!("{}", v);
    dbg!(r);
    sender.stop_server()?;
    t.join().unwrap();
    Ok(())
}
*/
