use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener},
    str::FromStr,
    sync::{Arc, atomic::AtomicBool},
};

use eagle_eye_broadcaster::SenderInfo;
use eagle_eye_http::{HttpResponse, Status};
use eagle_eye_jobs::{file::RemoveFile, ping_pong::Ping};
use eagle_eye_proto::{
    client::{ClientSync, TaskSenderSync},
    task::ExecuteResult,
};

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7575")?;
    for stream in listener.incoming() {
        let stream = stream?;
        let reader = BufReader::new(stream.try_clone().unwrap());
        let writer = BufWriter::new(stream);
        handle_stream(reader, writer)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Method {
    Get,
    Post,
}

impl FromStr for Method {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "GET" => Ok(Self::Get),
            "POST" => Ok(Self::Post),
            _ => Err(()),
        }
    }
}

fn handle_stream<R: io::Read + BufRead, W: io::Write>(
    mut reader: R,
    mut writer: W,
) -> io::Result<()> {
    let mut iter_line = reader.by_ref().lines();

    let first_line = iter_line
        .next()
        .unwrap()
        .unwrap()
        .split_whitespace()
        .map(|v| v.to_owned())
        .collect::<Vec<String>>();
    let method = first_line.get(0).unwrap().as_str();
    let method = Method::from_str(method);
    if method.is_err() {
        return Ok(());
    }
    let method = method.unwrap();
    let path = first_line.get(1).unwrap().as_str();
    let mut headers = HashMap::new();
    for line in iter_line {
        let line = line.unwrap();
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_owned(), value.trim().to_owned());
        }
    }

    let client = ClientSync::new();
    let mut v = client
        .connect::<512>([0; 32], "127.69.69.69:6969".parse().unwrap())
        .unwrap();
    let r = v.send(RemoveFile::new("my-file.txt".into()), &mut writer);
    match path {
        "/" => HttpResponse::new().send_file(&mut writer, "web/index.html")?,
        "/scan" => {}
        _ => HttpResponse::new()
            .status(Status::NotFound)
            .send_file(&mut writer, "web/404.html")?,
    }
    Ok(())
}

struct MyDevices<const N: usize, R: io::Read, W: io::Write> {
    client: ClientSync,
    // online devices
    online: Vec<(u128, TaskSenderSync<N, R, W>)>,
    // u128: device id
    // [u8; 32]: password
    all: Vec<(u128, [u8; 32])>,
}

impl<const N: usize, R: io::Read, W: io::Write> MyDevices<N, R, W> {
    pub fn refresh_online_devices(&mut self) {
        let mut offline_device_index = Vec::new();
        let mut iter_online = self.online.iter_mut().map(|(_, v)| v).enumerate();
        while let Some((i, t)) = iter_online.next() {
            match t.send(Ping, std::io::sink()) {
                Ok(ExecuteResult::Ok) => continue,
                _ => offline_device_index.push(i),
            }
        }
        for index in offline_device_index {
            self.online.remove(index);
        }
    }
    pub fn scan(&mut self) -> io::Result<()> {
        self.refresh_online_devices();
        let online_device_id = self.online.iter().map(|(id, _)| *id).collect::<Vec<u128>>();
        let mut iter_all_device = self.all.iter();
        let listener = TcpListener::bind(SocketAddr::new(
            std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
            0,
        ))?;
        let is_running = Arc::new(AtomicBool::new(true));
        while let Some((id, pass)) = iter_all_device.next() {
            if online_device_id.contains(id) {
                continue;
            }
            let v = SenderInfo::builder()
                .is_running(is_running.clone())
                .prefix(":eagle-eye:")
                .data(id.to_be_bytes())
                .data(listener.local_addr()?.port().to_be_bytes())
                .broadcast_addr(SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)),
                    6923,
                ))
                .socket_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0))
                .build();
            let t1 = std::thread::spawn(move || v.send());
            match listener.accept() {
                Ok((stream, addr)) => {}
                Err(_) => {}
            }
            t1.join().unwrap()?;
        }
        todo!()
    }
}
