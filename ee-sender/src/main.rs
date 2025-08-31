/*

use tokio::io;

use std::{convert::Infallible, io, net::SocketAddr};
use http_body_util::{BodyExt, Full};
use hyper::{Request, Response, body::Bytes, server::conn::http1, service::service_fn};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));

    let listener = TcpListener::bind(addr).await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(hello))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn hello(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from("Hello, World!"))))
}
*/
use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, BufWriter, Write},
    mem::ManuallyDrop,
    net::{TcpListener, TcpStream},
    str::FromStr,
    time::Duration,
};

use ee_device::{ClientSync, Device, DeviceManager};
use ee_http::{HttpRequest, HttpResponse, Method, Status};
use ee_task::{GetId, file::RemoveFileSync, prelude::Ping};

fn main() -> io::Result<()> {
    let client = ClientSync::new().device_connect_time_out(Duration::from_secs(3));
    let mut my_devices = DeviceManager::new();
    my_devices.push_device(Device::new().id(123).key([33; 32]));
    my_devices.push_device(Device::new().id(3).key([0; 32]));
    my_devices.push_device(Device::new().id(10).key([17; 32]));
    my_devices.push_device(Device::new().id(15).key([123; 32]));

    let listener = TcpListener::bind("0.0.0.0:8080")?;
    for stream in listener.incoming() {
        let stream = stream?;
        stream.set_read_timeout(Some(Duration::from_secs(3)))?;
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        loop {
            match handle(&client, &mut reader, &mut writer, &mut my_devices) {
                Ok(true) => continue,
                Ok(false) => break,
                Err(err) => {
                    eprintln!("{err}\n");
                    break;
                }
            }
        }
    }
    Ok(())
}

fn my_print<T: std::fmt::Display>(v: T) {
    let mut stdout = std::io::stdout();
    let _ = writeln!(stdout, "{v}");
    let _ = stdout.flush();
}

fn handle(
    client: &ClientSync,
    reader: &mut BufReader<TcpStream>,
    writer: &mut BufWriter<TcpStream>,
    manager: &mut DeviceManager,
) -> io::Result<bool> {
    let mut req = get_http_request(reader)?;
    my_print(format!("{:?}", &req));
    match req.get_path() {
        "/" => HttpResponse::new().send_file(writer, "web/index.html")?,
        "/api/scan-devices" => {
            manager.scan(client)?;
            let online = manager
                .get_online_device()
                .iter()
                .map(|&v| {
                    let mut h = HashMap::new();
                    h.insert("user", v.get_user().to_owned());
                    h.insert("os", v.get_os().to_owned());
                    h.insert("id", v.get_id().to_string());
                    h
                })
                .collect::<Vec<HashMap<&'static str, String>>>();
            HttpResponse::new().send_json_str(writer, serde_json::to_string(&online).unwrap())?;
        }
        "/api" | "/api/" => {
            handle_api(client, &mut req, writer, manager)?;
        }
        _ => HttpResponse::default()
            .status(Status::NotFound)
            .send_file(writer, "web/404.html")?,
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
    let method = Method::from_str(t.first().unwrap()).unwrap();
    let path = *t.get(1).unwrap();
    let version = *t.get(2).unwrap();
    let mut headers = HashMap::new();
    for line in iter {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            break;
        }
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

fn handle_api(
    client: &ClientSync,
    req: &mut HttpRequest,
    writer: &mut BufWriter<TcpStream>,
    manager: &mut DeviceManager,
) -> io::Result<()> {
    let task_id = req.get_header("TaskId");
    if task_id.is_none() {
        return HttpResponse::new().send_str(writer, "TaskId not found.");
    }
    let device_id = match req.get_header("Id") {
        Some(id) => id.parse::<u128>().unwrap(),
        None => return HttpResponse::new().send_str(writer, "Id not found."),
    };
    match task_id.unwrap() {
        v if v == Ping::id() => {
            if manager
                .send(client, &device_id, req, writer, Ping::new())
                .is_ok()
            {
                return HttpResponse::new().send_str(writer, "pong");
            } else {
                return HttpResponse::new().send_str(writer, "Device is not online.");
            }
        }
        v if v == RemoveFileSync::id() => {}
        _ => {}
    }
    todo!()
}
