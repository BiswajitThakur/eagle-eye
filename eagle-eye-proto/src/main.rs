use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
};

use eagle_eye_proto::{
    FlowControl,
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
    fn execute_on_receiver<T: io::Read + io::Write>(mut stream: T) -> io::Result<(FlowControl, T)> {
        stream.write_all(&FlowControl::Continue.to_be_bytes())?;
        stream.write_all(b"/path/home\n")?;
        stream.write_all(&ExecuteResult::Ok.to_be_bytes())?;
        stream.flush()?;
        Ok((FlowControl::Continue, stream))
    }
}

const KEY: [u8; 32] = [1; 32];

fn main() -> io::Result<()> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 69, 69, 69)), 6969);
    let t = std::thread::spawn(move || {
        let mut registery = TaskRegisterySync::new_default(handler);
        registery.register("pwd", GetPwd::execute_on_receiver);
        let l = TcpListener::bind(addr).unwrap();
        let listener: EagleEyeServerSync<EagleEyeStreamSync<1014, _, _>> =
            EagleEyeServerSync::new(KEY, registery).set_log_path("error.log");
        listener
            .run(l.incoming().map(|v| v.map(|m| (m.try_clone().unwrap(), m))))
            .unwrap();
    });
    std::thread::sleep(std::time::Duration::from_secs(1));
    let client = ClientSync::<1024>::new();
    let mut sender = client.connect(KEY, addr)?;
    let mut output = Vec::new();
    // <id>\n<flow>
    let r = sender.send(GetPwd::new(), &mut output, io::sink())?;
    let v = unsafe { String::from_utf8_unchecked(output) };
    println!("{}", v);
    dbg!(r);
    sender.stop_server()?;
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
