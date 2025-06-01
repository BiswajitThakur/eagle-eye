use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{self, Duration, Instant},
};

use eagle_eye_broadcaster::{ReceiverInfo, SenderInfo};

#[test]
#[allow(unused_assignments)]
fn test_is_running_sender() {
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
    let mut f = true;
    let mut f1 = false;
    let mut f2 = false;
    loop {
        std::thread::sleep(Duration::from_millis(50));
        if now.elapsed() > Duration::from_secs(5) {
            f1 = true;
            assert!(v.is_finished());
            break;
        }
        if now.elapsed() < Duration::from_secs(3) {
            f2 = true;
            assert!(!v.is_finished());
        }
        if f && now.elapsed() > Duration::from_secs(3) && now.elapsed() < Duration::from_secs(5) {
            f = false;
            assert!(!v.is_finished());
            is_running.store(false, Ordering::Relaxed);
        }
    }
    assert!(!f);
    assert!(f1);
    assert!(f2);
    v.join().unwrap().unwrap();
}

#[test]
fn test_is_running_receiver() {
    let socket = UdpSocket::bind("127.0.0.1:7755").unwrap();
    socket
        .set_read_timeout(Some(Duration::from_millis(200)))
        .unwrap();
    let is_running = Arc::new(AtomicBool::new(true));
    let receiver = ReceiverInfo::<7>::builder()
        .prefix("hello")
        .is_running(is_running.clone())
        .socket(socket)
        .build();
    let t = { std::thread::spawn(move || for _ in receiver {}) };
    let now = Instant::now();
    let mut f = true;
    loop {
        if f && now.elapsed() > Duration::from_secs(3) && now.elapsed() < Duration::from_secs(5) {
            f = false;
            assert!(!t.is_finished());
            is_running.store(false, Ordering::Relaxed);
        }
        if now.elapsed() > Duration::from_secs(5) {
            assert!(t.is_finished());
            break;
        }
    }
    t.join().unwrap();
}
