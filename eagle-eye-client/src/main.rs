use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, BufWriter, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    str::FromStr,
    sync::{Arc, Mutex, atomic::AtomicBool},
    time::Duration,
};

use aes::cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
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

#[derive(Debug, Clone, PartialEq, Default)]
struct Device {
    id: u128,
    password: [u8; 32],
    os: String,
    user: String,
}

impl Device {
    pub fn id(mut self, id: u128) -> Self {
        self.id = id;
        self
    }
    pub fn password(mut self, password: [u8; 32]) -> Self {
        self.password = password;
        self
    }
    pub fn os<T: Into<String>>(mut self, os: T) -> Self {
        self.os = os.into();
        self
    }
    pub fn user<T: Into<String>>(mut self, user: T) -> Self {
        self.user = user.into();
        self
    }
    pub fn to_vec(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&self.id.to_be_bytes());
        v.extend_from_slice(&self.password);
        let os = self.os.as_bytes();
        let os_len: u16 = os.len() as u16;
        v.extend_from_slice(&os_len.to_be_bytes());
        v.extend_from_slice(os);
        let user = self.user.as_bytes();
        let user_len: u16 = user.len() as u16;
        v.extend_from_slice(&user_len.to_be_bytes());
        v.extend_from_slice(user);
        v
    }
    pub fn from_reader<R: io::Read>(mut reader: R) -> io::Result<Self> {
        let mut buf = [0; 16];
        reader.read_exact(&mut buf)?;
        let id = u128::from_be_bytes(buf);
        let mut password = [0; 32];
        reader.read_exact(&mut password)?;
        reader.read_exact(&mut buf[0..2])?;
        let mut os_len = u16::from_be_bytes([buf[0], buf[1]]) as usize;
        let mut os = String::new();
        loop {
            if os_len == 0 {
                break;
            }
            let r = reader.read(&mut buf[0..std::cmp::min(16, os_len)])?;
            os.push_str(unsafe { std::str::from_utf8_unchecked(&buf[0..r]) });
            os_len -= r;
        }
        reader.read_exact(&mut buf[0..2])?;
        let mut user_len = u16::from_be_bytes([buf[0], buf[1]]) as usize;
        let mut user = String::new();
        loop {
            if user_len == 0 {
                break;
            }
            let r = reader.read(&mut buf[0..std::cmp::min(16, user_len)])?;
            user.push_str(unsafe { std::str::from_utf8_unchecked(&buf[0..r]) });
            user_len -= r;
        }
        Ok(Self {
            id,
            password,
            os,
            user,
        })
    }
    pub fn get_id(&self) -> &u128 {
        &self.id
    }
    pub fn get_password(&self) -> &[u8; 32] {
        &self.password
    }
    pub fn get_os(&self) -> &str {
        self.os.as_str()
    }
    pub fn get_user(&self) -> &str {
        self.user.as_str()
    }
}

struct MyDevices<const N: usize, R: io::Read, W: io::Write> {
    client: ClientSync,
    // online devices
    online: Vec<(u128, TaskSenderSync<N, R, W>)>,
    // u128: device id
    // [u8; 32]: password
    all: Vec<Device>,
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
    pub fn get_online_device_id(&self) -> Vec<u128> {
        self.online.iter().map(|(id, _)| *id).collect::<Vec<u128>>()
    }
    pub fn scan(&mut self) -> io::Result<()> {
        type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
        let client = ClientSync::new();
        let mut buffer = [0; 256];
        self.refresh_online_devices();
        let online_device_id = self.get_online_device_id();
        let mut iter_all_device = self.all.iter();
        let is_running = Arc::new(AtomicBool::new(true));
        while let Some(device) = iter_all_device.next() {
            if online_device_id.contains(device.get_id()) {
                continue;
            }
            let listener = TcpListener::bind(SocketAddr::new(
                std::net::IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
                0,
            ))?;
            let addr = listener.local_addr()?;
            let listener_port = addr.port();
            let iv = rand::random::<[u8; 16]>();
            let secret = rand::random::<[u8; 16]>();
            let ct = Aes256CbcEnc::new(device.get_password().into(), &iv.into());
            let id_bytes = &device.get_id().to_be_bytes(); // 16 bytes = 128 bit
            buffer[..16].copy_from_slice(id_bytes);
            buffer[16..].copy_from_slice(&secret);
            buffer[32..].copy_from_slice(&listener_port.to_be_bytes());
            let encrypted = ct.encrypt_padded_mut::<Pkcs7>(&mut buffer, 34).unwrap();
            let enc_len = encrypted.len() as u16;

            let v = SenderInfo::builder()
                .is_running(is_running.clone())
                .prefix(":eagle-eye:")
                .data(&iv)
                .data(enc_len.to_be_bytes())
                .data(encrypted)
                .broadcast_addr(SocketAddr::new(
                    IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255)),
                    6923,
                ))
                .socket_addr(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0))
                .build();
            let (send, recv) = std::sync::mpsc::channel::<TcpStream>();
            let t1 = std::thread::spawn(move || v.send());
            let t2 = std::thread::spawn(move || {
                let mut buf = [0; 16];
                for stream in listener.incoming() {
                    if stream.is_err() {
                        break;
                    }
                    let mut stream = stream.unwrap();
                    if stream
                        .set_read_timeout(Some(Duration::from_secs(1)))
                        .is_err()
                    {
                        break;
                    }
                    if stream.read_exact(&mut buf[0..1]).is_err() {
                        continue;
                    }
                    if buf[0] == 111 {
                        break;
                    }
                    if stream.read_exact(&mut buf).is_err() {
                        continue;
                    }
                    if buf == secret {
                        send.send(stream).unwrap();
                        break;
                    }
                }
            });

            let now = std::time::Instant::now();
            loop {
                std::thread::sleep(Duration::from_millis(200));
                if let Ok(stream) = recv.try_recv() {
                    // let t = client.connect(key, addr);
                    break;
                }
                if now.elapsed() > Duration::from_secs(5) {
                    let mut v = TcpStream::connect(addr)?;
                    v.write_all(&[111])?;
                    break;
                }
            }

            t1.join().unwrap()?;
            t2.join().unwrap();
        }
        todo!()
    }
}
