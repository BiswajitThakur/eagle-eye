/*
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

use eagle_eye_broadcaster::SenderInfo;

#[test]
#[ignore = "reason"]
fn is_broadcasting() {
    let broadcaster_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)), 5437);
    let socket_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0);
    let is_running = Arc::new(AtomicBool::new(true));
    let prefix = "----->>";
    let sender = SenderInfo::builder()
        .prefix(prefix)
        .send_addr(broadcaster_addr)
        .socket_addr(socket_addr)
        .interval(Arc::new(AtomicU64::new(200)))
        .build();
    let h = {
        let is_running_clone = is_running.clone();
        std::thread::spawn(move || sender.send(is_running_clone))
    };
    std::thread::sleep(Duration::from_millis(100));
    let socket = UdpSocket::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        broadcaster_addr.port(),
    ))
    .unwrap();
    socket
        .set_read_timeout(Some(Duration::from_secs(5)))
        .unwrap();
    let mut buf = [0; 10];
    match socket.recv_from(&mut buf) {
        Ok(_) => {
            assert!(buf.starts_with(prefix.as_bytes()));
            is_running.store(false, Ordering::Relaxed);
        }
        Err(_) => panic!("Should not return errr"),
    }
    h.join().unwrap().unwrap();
}
*/
