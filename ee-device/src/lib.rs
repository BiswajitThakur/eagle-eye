mod task_sender;
mod utils;

pub use task_sender::TaskSenderSync;

use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener, TcpStream},
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

use aes::cipher::{BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use ee_broadcaster::SenderInfo;
use ee_task::{ExecuteResult, ping_pong::Ping};

use crate::utils::handle_auth_on_sender_sync;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    id: u128,
    key: [u8; 32],
    os: String,
    user: String,
}

impl Default for Device {
    fn default() -> Self {
        let os = if cfg!(target_os = "linux") {
            "Linux"
        } else if cfg!(target_os = "windows") {
            "Windows"
        } else if cfg!(target_os = "macos") {
            "macOS"
        } else if cfg!(target_os = "freebsd") {
            "FreeBSD"
        } else if cfg!(target_os = "dragonfly") {
            "DragonFly BSD"
        } else if cfg!(target_os = "netbsd") {
            "NetBSD"
        } else if cfg!(target_os = "openbsd") {
            "OpenBSD"
        } else if cfg!(target_os = "android") {
            "Android"
        } else if cfg!(target_os = "ios") {
            "iOS"
        } else {
            "Unknown"
        }
        .to_owned();
        Self {
            id: 0,
            key: [0; 32],
            user: "Unknown".to_owned(),
            os,
        }
    }
}

impl Device {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn id(mut self, id: u128) -> Self {
        self.id = id;
        self
    }
    pub fn key(mut self, key: [u8; 32]) -> Self {
        self.key = key;
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
    pub fn save<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        let os = self.os.as_bytes();
        let os_len = os.len() as u16;
        let user = self.user.as_bytes();
        let user_len = user.len() as u16;
        writer.write_all(&os_len.to_be_bytes())?;
        writer.write_all(&user_len.to_be_bytes())?;
        writer.write_all(&self.id.to_be_bytes())?;
        writer.write_all(&self.key)?;
        writer.write_all(os)?;
        writer.write_all(user)?;
        writer.write_all(&[111, 0])?;
        writer.flush()
    }
    pub fn from_reader<R: io::Read>(mut reader: R) -> io::Result<Self> {
        let mut buf = [0; 16];
        let mut small = [0; 2];
        reader.read_exact(&mut small)?;
        let mut os_len = u16::from_be_bytes([small[0], small[1]]) as usize;
        reader.read_exact(&mut small)?;
        let mut user_len = u16::from_be_bytes([small[0], small[1]]) as usize;
        if os_len + user_len > 128 {
            return Err(io::Error::other(
                "ERROR: length of (user + os) can not be greater then 128",
            ));
        }
        reader.read_exact(&mut buf)?;
        let id = u128::from_be_bytes(buf);
        let mut key = [0; 32];
        reader.read_exact(&mut key)?;
        reader.read_exact(&mut buf[0..2])?;
        let mut os = String::new();
        loop {
            if os_len == 0 {
                break;
            }
            let r = reader.read(&mut buf[0..std::cmp::min(16, os_len)])?;
            os.push_str(unsafe { std::str::from_utf8_unchecked(&buf[0..r]) });
            os_len -= r;
        }
        let mut user = String::new();
        loop {
            if user_len == 0 {
                break;
            }
            let r = reader.read(&mut buf[0..std::cmp::min(16, user_len)])?;
            user.push_str(unsafe { std::str::from_utf8_unchecked(&buf[0..r]) });
            user_len -= r;
        }
        Ok(Self { id, key, os, user })
    }
    pub fn get_id(&self) -> &u128 {
        &self.id
    }
    pub fn get_key(&self) -> &[u8; 32] {
        &self.key
    }
    pub fn get_os(&self) -> &str {
        self.os.as_str()
    }
    pub fn get_user(&self) -> &str {
        self.user.as_str()
    }
}

pub struct ClientSync {
    id: u128,
    // devices: Vec<Device>,
    log: Option<PathBuf>,
}

impl Default for ClientSync {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientSync {
    pub fn new() -> Self {
        Self {
            id: 0,
            // devices: Vec::new(),
            log: None,
        }
    }
    pub fn log<T: Into<PathBuf>>(mut self, path: T) -> Self {
        self.log = Some(path.into());
        self
    }
    pub fn connect<const N: usize>(
        &self,
        key: [u8; 32],
        stream: TcpStream,
    ) -> io::Result<TaskSenderSync<N, TcpStream, TcpStream>> {
        let stream1 = stream;
        let stream2 = stream1.try_clone()?;
        let e_stream = match handle_auth_on_sender_sync::<N, _, _>(key, stream1, stream2)? {
            Some(v) => v,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "Wrong Password",
                ));
            }
        };
        Ok(TaskSenderSync::new(e_stream))
    }
}

pub struct DeviceManager<const N: usize> {
    client: ClientSync,
    // online devices
    online: HashMap<u128, TaskSenderSync<N, TcpStream, TcpStream>>,
    // u128: device id
    // [u8; 32]: password
    all: Vec<Device>,
}

impl<const N: usize> Default for DeviceManager<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> DeviceManager<N> {
    pub fn new() -> Self {
        Self {
            client: ClientSync::new(),
            online: HashMap::new(),
            all: Vec::new(),
        }
    }
    pub fn from_reader<R: io::Read>(mut reader: R) -> io::Result<Self> {
        let mut devices = Vec::new();
        let mut buf = [0; 2];
        reader.read_exact(&mut buf)?;
        let mut total_device = u16::from_be_bytes(buf);
        loop {
            if total_device == 0 {
                break;
            }
            let device = Device::from_reader(&mut reader)?;
            devices.push(device);
            total_device -= 1;
        }
        Ok(Self {
            client: ClientSync::new(),
            online: HashMap::new(),
            all: devices,
        })
    }
    pub fn save<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        let total = self.all.len() as u16;
        writer.write_all(&total.to_be_bytes())?;
        for device in self.all.iter() {
            device.save(&mut writer)?;
        }
        Ok(())
    }
    pub fn push_device(&mut self, device: Device) {
        let index_if_exists = self
            .all
            .iter()
            .enumerate()
            .find(|(_, v)| v.get_id() == device.get_id())
            .map(|(i, _)| i);
        if let Some(index) = index_if_exists {
            self.all[index] = device
        } else {
            self.all.push(device);
        }
    }
    pub fn total_online(&self) -> usize {
        self.online.len()
    }
    pub fn total_device(&self) -> usize {
        self.all.len()
    }
    pub fn refresh_online_devices(&mut self) {
        let mut offline_device = Vec::new();
        let iter_online = self.online.iter_mut();
        for (id, t) in iter_online {
            match t.send(Ping, std::io::sink()) {
                Ok(ExecuteResult::Ok) => continue,
                _ => offline_device.push(*id),
            }
        }
        for id in offline_device {
            self.online.remove(&id);
        }
    }
    pub fn get_online_device_id(&self) -> Vec<u128> {
        self.online.keys().copied().collect::<Vec<u128>>()
    }
    pub fn scan(&mut self) -> io::Result<()> {
        type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
        let client = ClientSync::new();
        let mut buffer = [0; 256];
        self.refresh_online_devices();
        let online_device_id = self.get_online_device_id();
        let iter_all_device = self.all.iter();
        let is_running = Arc::new(AtomicBool::new(true));
        for device in iter_all_device {
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
            let ct = Aes256CbcEnc::new(device.get_key().into(), &iv.into());
            let id_bytes = &device.get_id().to_be_bytes(); // 16 bytes = 128 bit
            buffer[..16].copy_from_slice(id_bytes);
            buffer[16..32].copy_from_slice(&secret);
            buffer[32..34].copy_from_slice(&listener_port.to_be_bytes());
            let encrypted = ct.encrypt_padded_mut::<Pkcs7>(&mut buffer, 34).unwrap();
            let data_len = encrypted.len() as u16 + iv.len() as u16;

            is_running.store(true, std::sync::atomic::Ordering::Relaxed);
            let v = SenderInfo::builder()
                .is_running(is_running.clone())
                .prefix(":eagle-eye:")
                .data(data_len.to_be_bytes())
                .data(iv)
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
                    is_running.store(false, std::sync::atomic::Ordering::Relaxed);
                    let t = client.connect::<N>(device.key, stream)?;
                    self.online.insert(*device.get_id(), t);
                    break;
                }
                if now.elapsed() > Duration::from_secs(5) {
                    // stop t1 and t2 thread
                    is_running.store(false, std::sync::atomic::Ordering::Relaxed);
                    let mut v = TcpStream::connect(addr)?;
                    v.write_all(&[111])?;
                    break;
                }
            }
            t1.join().unwrap()?;
            t2.join().unwrap();
        }
        Ok(())
    }
}
