use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
};

use ee_device::{ClientSync, Device, DeviceManager};
use ee_http::{HttpRequest, HttpResponse, Method};
use ee_task::file::RemoveFile;

fn main() -> io::Result<()> {
    let client = ClientSync::new();
    let mut my_devices = DeviceManager::<512>::new();
    my_devices.push_device(Device::new().id(123).key([33; 32]));
    println!(
        "No of online Devices: {}/{}",
        my_devices.total_online(),
        my_devices.total_device()
    );
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    for stream in listener.incoming() {
        let stream = stream?;
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        loop {
            match handle::<512>(&client, &mut reader, &mut writer, &mut my_devices) {
                _ => break,
            }
        }
    }
    Ok(())
}

fn my_print<T: std::fmt::Display>(v: T) {
    let mut stdout = std::io::stdout();
    let _ = writeln!(stdout, "{}", v);
    let _ = stdout.flush();
}

fn handle<const N: usize>(
    client: &ClientSync,
    reader: &mut BufReader<TcpStream>,
    writer: &mut BufWriter<TcpStream>,
    manager: &mut DeviceManager<N>,
) -> io::Result<bool> {
    let mut req = get_http_request(reader)?;
    my_print(req.get_path());
    match req.get_path() {
        "/scan" => {
            manager.scan(client)?;
            HttpResponse::new().send_str(writer, "success")?;
        }
        "/online_device" => {}
        v if v.starts_with("/send/") => {
            let id = v.strip_prefix("/send/").unwrap();
            let v = manager.send(client, id, &mut req, &mut writer, RemoveFile::new("hello"));
        }
        _ => {}
    }
    Ok(req.get_header("Connection") == Some("keep-alive"))
}

fn read_n<R: io::Read>(mut reader: R, mut n: usize) -> io::Result<Vec<u8>> {
    const LEN: usize = 32;
    let mut buf = [0u8; LEN];
    let mut v = Vec::new();
    loop {
        if n == 0 {
            break;
        }
        let r = reader.read(&mut buf[0..std::cmp::min(n, LEN)])?;
        if r == 0 {
            break;
        }
        v.extend_from_slice(&buf[0..r]);
        n -= r;
    }
    Ok(v)
}

fn get_http_request(reader: &mut BufReader<TcpStream>) -> io::Result<HttpRequest> {
    let mut iter = reader.lines();
    let first_line = iter.next().unwrap()?;
    let t: Vec<&str> = first_line.split_whitespace().collect();
    let method = Method::from_str(t.get(0).unwrap()).unwrap();
    let path = *t.get(1).unwrap();
    let version = *t.get(2).unwrap();
    let mut headers = HashMap::new();
    for line in iter {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            break;
        }
        my_print(line);
        let v = line
            .split_once(':')
            .map(|(x, y)| (x.trim(), y.trim()))
            .unwrap();
        headers.insert(v.0.to_owned(), v.1.to_owned());
    }
    let body;
    if let Some(v) = headers.get("Content-Length") {
        let len: usize = v.parse().unwrap();
        let u = read_n(reader, len)?;
        body = Some(u);
    } else {
        body = None;
    }
    Ok(HttpRequest::default()
        .method(method)
        .path(path)
        .version(version)
        .headers(headers)
        .body(body))
}
