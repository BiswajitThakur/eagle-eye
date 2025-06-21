use std::{
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
};

use eagle_eye_proto::{
    FlowControl,
    listener::EagleEyeListenerSync,
    stream::EagleEyeStreamSync,
    task::{ExecuteResult, TaskRegisterySync, TaskSync},
    utils::handle_auth_on_client_sync,
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
    fn execute(&self, stream: T, _ok: W, _err: E) -> io::Result<ExecuteResult> {
        Ok(ExecuteResult::Ok)
    }
}

const KEY: [u8; 32] = [1; 32];

fn main() -> io::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 69, 69, 69)), 6969);
    let t = std::thread::spawn(move || {
        let l = TcpListener::bind(addr).unwrap();
        let listener: EagleEyeListenerSync<EagleEyeStreamSync<1014, _, _>> =
            EagleEyeListenerSync::new(KEY, TaskRegisterySync::new(handler))
                .set_log_path("error.log");
        listener
            .run_server(l.incoming().map(|v| v.map(|m| (m.try_clone().unwrap(), m))))
            .unwrap();
    });
    std::thread::sleep(std::time::Duration::from_secs(1));
    let stream = TcpStream::connect(addr).unwrap();
    let mut stream = handle_auth_on_client_sync::<512>(KEY, &stream)
        .unwrap()
        .unwrap();
    let mut output = Vec::new();
    // <id>\n<flow>
    let r = stream.send_task(GetPwd::new(), &mut output, io::sink())?;
    dbg!(r);
    //stream.send_task(GetPwd::new(), &mut output, io::sink())?;
    //stream.end()?;
    stream.stop_server()?;
    t.join().unwrap();
    Ok(())
}

fn handler<T: io::Read + io::Write>(mut stream: T) -> io::Result<(FlowControl, T)> {
    let mut buf = [0; 1];
    stream.read_exact(&mut buf)?;
    let flow = FlowControl::try_from(buf);
    if flow.is_err() {
        stream.write_all(&FlowControl::Close.to_be_bytes())?;
        stream.write_all(&ExecuteResult::InvalidRequest.to_be_bytes())?;
        return Ok((FlowControl::Close, stream));
    }
    let flow = flow.unwrap();
    stream.write_all(&FlowControl::Close.to_be_bytes())?;
    stream.write_all(&ExecuteResult::UnknownTask.to_be_bytes())?;
    stream.flush()?;
    Ok((flow, stream))
}
