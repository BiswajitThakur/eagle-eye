use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{self, Duration},
};

use eagle_eye_broadcaster::SenderInfo;

#[test]
fn test_running() {
    let interval = Arc::new(AtomicU64::new(200));
    let sender = SenderInfo::builder()
        .prefix("hello".into())
        .send_addr(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 96, 69, 96)),
            6969,
        ))
        .interval(interval)
        .socket_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0))
        .broadcast_addr(SocketAddr::new(
            IpAddr::V4(Ipv4Addr::new(127, 255, 255, 255)),
            7766,
        ))
        .build();
    let is_running = Arc::new(AtomicBool::new(true));
    let v = {
        let is_running_clone = Arc::clone(&is_running);
        std::thread::spawn(move || sender.send(is_running_clone))
    };
    let now = time::Instant::now();
    loop {
        std::thread::sleep(Duration::from_millis(50));
        if now.elapsed() > Duration::from_secs(5) {
            assert!(v.is_finished());
        }
        if now.elapsed() < Duration::from_secs(3) {
            assert!(!v.is_finished());
        }
        if now.elapsed() > Duration::from_secs(3) && now.elapsed() < Duration::from_secs(5) {
            assert!(!v.is_finished());
            is_running.store(false, Ordering::Relaxed);
            break;
        }
    }
    v.join().unwrap().unwrap();
}
