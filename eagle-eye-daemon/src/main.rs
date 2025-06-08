mod job;

use std::{
    io::{self, ErrorKind},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64},
    },
    thread::JoinHandle,
    time::Duration,
};

use eagle_eye_broadcaster::SenderInfo;

const UDP_SOCKET_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
const BROADCAST_ADDR: SocketAddr =
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), 7753);

fn main() -> io::Result<()> {
    let server = TcpListener::bind("0.0.0.0:0")?;
    let server_addr = server.local_addr()?;
    dbg!(server_addr);
    let is_running_broadcaster = Arc::new(AtomicBool::new(true));
    let send_interval = Arc::new(AtomicU64::new(1000));
    let h1 = broadcaster(
        is_running_broadcaster.clone(),
        send_interval.clone(),
        server_addr,
    );

    h1.join().unwrap()?;
    Ok(())
}

fn broadcaster(
    is_running: Arc<AtomicBool>,
    interval: Arc<AtomicU64>,
    addr: SocketAddr,
) -> JoinHandle<io::Result<()>> {
    let sender = SenderInfo::builder()
        .socket_addr(UDP_SOCKET_ADDR)
        .broadcast_addr(BROADCAST_ADDR)
        .prefix("EagleEYE")
        .send_addr(addr)
        .interval(interval)
        .build();
    std::thread::spawn(move || {
        let sender = sender;
        let is_running = is_running;
        loop {
            match sender.send(is_running.clone()) {
                Ok(_) => break,
                Err(err) if err.kind() == ErrorKind::NetworkUnreachable => {
                    #[cfg(debug_assertions)]
                    {
                        dbg!(err);
                        std::thread::sleep(Duration::from_secs(3));
                    }
                    #[cfg(not(debug_assertions))]
                    {
                        std::thread::sleep(Duration::from_secs(15));
                    }
                    continue;
                }
                err @ Err(_) => {
                    #[cfg(debug_assertions)]
                    {
                        dbg!(&err);
                    }
                    return err;
                }
            }
        }
        Ok(())
    })
}
