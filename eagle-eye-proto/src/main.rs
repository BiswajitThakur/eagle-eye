use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream},
};

use eagle_eye_proto::{
    Connection,
    listener::EagleEyeListenerSync,
    stream::EagleEyeStreamSync,
    task::{ExecuteResult, TaskSync},
    utils::handle_stream_client_sync,
};

struct GetPwd {}

impl GetPwd {
    fn new() -> Self {
        Self {}
    }
}

impl<T: io::Read + io::Write, W: io::Write, E: io::Write> TaskSync<T, W, E> for GetPwd {
    fn id() -> &'static str {
        "pwd"
    }
    fn execute(&self, mut stream: T, _ok: W, _err: E) -> io::Result<ExecuteResult> {
        stream.write_all(<Self as TaskSync<T, W, E>>::id().as_bytes())?;
        stream.write_all(b"\n")?;

        Ok(ExecuteResult::Ok)
    }
}

const KEY: [u8; 32] = [1; 32];

fn main() -> io::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 69, 69, 69)), 6969);
    let t = std::thread::spawn(move || {
        let listener = EagleEyeListenerSync::new(KEY).set_log_path("error.log");
        listener.run::<1024>(addr, handler).unwrap();
    });
    std::thread::sleep(std::time::Duration::from_secs(1));
    let stream = TcpStream::connect(addr).unwrap();
    let mut stream = handle_stream_client_sync::<512>(KEY, &stream)
        .unwrap()
        .unwrap();
    let mut output = Vec::new();
    stream.send_task(GetPwd::new(), &mut output, io::sink())?;
    stream.send_task(GetPwd::new(), &mut output, io::sink())?;
    stream.end()?;
    t.join().unwrap();
    Ok(())
}

fn handler<const N: usize>(
    stream: &mut EagleEyeStreamSync<N, &TcpStream, &TcpStream>,
) -> io::Result<Connection> {
    hh(stream)
}

fn hh<T: io::Read + io::Write>(stream: T) -> io::Result<Connection> {
    todo!()
}
