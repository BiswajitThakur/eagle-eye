use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use eagle_eye_proto::{
    job::{Connection, JobsSync},
    listener::sync::EagleEyeListenerSync,
};

const KEY: [u8; 32] = [1; 32];

fn main() -> io::Result<()> {
    let listener = EagleEyeListenerSync::new(KEY).set_log_path("erroe.log".into());
    listener.run::<512>(
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 69, 69, 69)), 6969),
        jobs(),
    )?;
    Ok(())
}

fn jobs<const N: usize>() -> JobsSync<N> {
    // fn(&mut EagleEyeStreamSync<N, &TcpStream, &TcpStream>) -> io::Result<Connection>,
    let mut jbs = JobsSync::<N>::new();
    jbs.insert("pwd", |_| Ok(Connection::Close), |_| Ok(Connection::Close));
    jbs.insert("pwd_v2", |v| todo!(), |v| todo!());
    jbs
}
