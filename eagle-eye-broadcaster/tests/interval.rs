use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering},
    },
    time::{self, Duration},
};

use eagle_eye_broadcaster::{ReceiverInfo, SenderInfo};

#[test]
#[ignore = "reason"]
fn test_running() {
    let is_running_sender = Arc::new(AtomicBool::new(true));
    let is_running_receiver = Arc::new(AtomicBool::new(true));
    let interval = Arc::new(AtomicU64::new(200));
    let sender = SenderInfo::builder()
        .prefix("hello")
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
    let socket = UdpSocket::bind("127.0.0.1:7766").unwrap();
    let receiver = ReceiverInfo::<30>::builder()
        .prefix("hello")
        .is_running(is_running_receiver.clone())
        .socket(socket)
        .build();
    let v = {
        let is_running_sender_clone = Arc::clone(&is_running_sender);
        std::thread::spawn(move || sender.send(is_running_sender_clone))
    };
    let now = time::Instant::now();
    let count = Arc::new(AtomicU32::new(0));

    let t = {
        let count_clone = count.clone();
        std::thread::spawn(move || {
            for _ in receiver {
                count_clone.fetch_add(1, Ordering::Relaxed);
            }
        })
    };
    let mut f1 = true;
    loop {
        if f1 && now.elapsed() > Duration::from_secs(3) && now.elapsed() < Duration::from_secs(4) {
            f1 = false;
            let total_count = count.load(Ordering::Relaxed);
            //assert!(total_count > 10);
            //assert!(total_count < 22);
            break;
        }
    }
    assert!(!f1);
    v.join().unwrap().unwrap();
    t.join().unwrap();
}
