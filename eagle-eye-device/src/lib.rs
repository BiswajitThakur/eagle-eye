use std::{
    fmt,
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::{SocketAddr, TcpListener},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, atomic::AtomicBool},
};

use eagle_eye_broadcaster::SenderInfo;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Device {
    id: u128,
    key: [u8; 32],
    user_name: String,
    os: String,
}

impl fmt::Display for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = self.id.to_be_bytes();
        for byte in id {
            write!(f, "{:02x}", byte)?;
        }
        for byte in self.key {
            write!(f, "{:02x}", byte)?;
        }
        let user = self.user_name.as_bytes();
        let user_len = user.len() as u8;
        write!(f, "{:02x}", user_len)?;
        for byte in user {
            write!(f, "{:02x}", byte)?;
        }
        let os = self.os.as_bytes();
        let os_len = os.len() as u8;
        write!(f, "{:02x}", os_len)?;
        for byte in os {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl FromStr for Device {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_byte(s: &str, i: &mut usize) -> Result<u8, &'static str> {
            if *i + 2 > s.len() {
                return Err("Unexpected end of input");
            }
            let b = u8::from_str_radix(&s[*i..*i + 2], 16).map_err(|_| "Invalid hex digit")?;
            *i += 2;
            Ok(b)
        }
        let mut i = 0;

        // Parse u128 (16 bytes)
        let mut id_bytes = [0u8; 16];
        for b in &mut id_bytes {
            *b = parse_byte(s, &mut i)?;
        }
        let id = u128::from_be_bytes(id_bytes);

        // Parse key (32 bytes)
        let mut key = [0u8; 32];
        for b in &mut key {
            *b = parse_byte(s, &mut i)?;
        }

        // user_name length
        let user_len = parse_byte(s, &mut i)? as usize;

        // user_name bytes
        let mut user_bytes = Vec::with_capacity(user_len);
        for _ in 0..user_len {
            user_bytes.push(parse_byte(s, &mut i)?);
        }
        let user_name = String::from_utf8(user_bytes).map_err(|_| "Invalid UTF-8 in username")?;

        // os length
        let os_len = parse_byte(s, &mut i)? as usize;

        // os bytes
        let mut os_bytes = Vec::with_capacity(os_len);
        for _ in 0..os_len {
            os_bytes.push(parse_byte(s, &mut i)?);
        }
        let os = String::from_utf8(os_bytes).map_err(|_| "Invalid UTF-8 in OS")?;

        Ok(Device {
            id,
            key,
            user_name,
            os,
        })
    }
}

impl Device {
    pub fn id(mut self, id: u128) -> Self {
        self.id = id;
        self
    }
    pub fn get_id(&self) -> u128 {
        self.id
    }
    pub fn key(mut self, key: [u8; 32]) -> Self {
        self.key = key;
        self
    }
    pub fn get_key(&self) -> &[u8] {
        &self.key
    }
    pub fn user_name<T: Into<String>>(mut self, name: T) -> Self {
        self.user_name = name.into();
        self
    }
    pub fn get_user_name(&self) -> &str {
        self.user_name.as_str()
    }
    pub fn os<T: Into<String>>(mut self, os: T) -> Self {
        self.os = os.into();
        self
    }
    pub fn get_os(&self) -> &str {
        self.os.as_str()
    }
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
            user_name: "Unknown".to_owned(),
            os,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceManager {
    devices: Vec<Device>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }
    pub fn push_device(&mut self, device: Device) {
        self.devices.push(device);
    }
    pub fn get_device(&self, id: u128) -> Option<&Device> {
        self.devices.iter().find(|&v| v.id == id)
    }
    pub fn from_reader<R: io::Read>(r: R) -> io::Result<Self> {
        let mut devices = Vec::new();
        let reader = BufReader::new(r);
        for line in reader.lines() {
            if line.is_err() {
                return Err(io::Error::other("Invalid String"));
            }
            let line = match line {
                Ok(v) => v,
                Err(err) => return Err(err),
            };
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let device = Device::from_str(line);
            if device.is_err() {
                return Err(io::Error::other("Invalid Data"));
            }
            devices.push(device.unwrap());
        }
        Ok(Self { devices })
    }
    pub fn save<W: io::Write>(&self, w: W) -> io::Result<()> {
        let mut f = BufWriter::new(w);
        for device in self.devices.iter() {
            f.write(device.to_string().as_bytes())?;
            f.write(b"\n")?;
        }
        f.flush()
    }
    pub fn scan_devices(&self, is_running: Arc<AtomicBool>) -> Vec<(&Device, SocketAddr)> {
        let is_broadcasting = Arc::new(AtomicBool::new(true));
        let listener = TcpListener::bind("0.0.0.0:0").unwrap();
        for device in self.devices.iter() {
            let s = SenderInfo::builder()
                .prefix(":eagle-eye:")
                .data(device.id.to_be_bytes())
                .is_running(is_broadcasting.clone())
                .socket_addr(addr)
                .build();
            s.send().unwrap();
        }
        todo!()
    }
}

#[cfg(test)]
mod test;
