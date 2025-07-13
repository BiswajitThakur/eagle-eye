use std::{
    collections::HashMap,
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::TcpListener,
    str::FromStr,
    sync::{Arc, Mutex},
};

use eagle_eye_jobs::RemoveFile;
use eagle_eye_proto::{client::ClientSync, server, task::GetId};

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7575")?;
    for stream in listener.incoming() {
        let stream = stream?;
        let mut reader = BufReader::new(stream.try_clone().unwrap());
        let mut writer = BufWriter::new(stream);
        loop {
            let v = handle_stream(reader, writer)?;
            if v.is_none() {
                break;
            }
            let (r, w) = v.unwrap();
            reader = r;
            writer = w;
        }
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
    writer: W,
) -> io::Result<Option<(R, W)>> {
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
        return Ok(None);
    }
    let method = method.unwrap();
    let path = first_line.get(1).unwrap().as_str();
    let mut headers = HashMap::new();
    for line in iter_line {
        let line = line.unwrap();
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_owned(), value.trim().to_owned());
        }
    }
    let client = ClientSync::new();
    match path {
        "/" => handle_home(reader, writer, method),
        //"/scan" => {}
        _ => handle_not_found(reader, writer, method),
    }
    /*
    let mut stdout = std::io::stdout();
    let mut reader = BufReader::new(&mut stream);
    let mut lines = reader.lines();
    let line = lines.next().unwrap().unwrap();
    let mut first_line = line
        .split_whitespace()
        .skip(1)
        .next()
        .unwrap()
        .split("/")
        .skip(1);
    let addr = first_line.next().unwrap();
    println!("addr: {}", &addr);
    let id = first_line.next().unwrap();
    println!("id: {}", &id);
    let sender = match id {
        v if v == RemoveFile::id() => {
            let mut sender = client.connect([1; 32], addr.parse().unwrap())?;
            println!("connected");
            let path = lines.next().unwrap().unwrap();
            let mut writer = BufWriter::new(&mut stream);
            let job = RemoveFile::from(path);
            let r = sender.send(job, io::sink(), io::sink())?;
            writeln!(writer, "HTTP/1.1 200 OK\r")?;
            writeln!(writer, "Content-Type: text/html\r")?;
            writeln!(writer, "\r")?;
            sender.end()?;
            if r.is_success() {
                writeln!(writer, "<h1>success</h1>")?;
                println!("success");
            } else {
                writeln!(writer, "<h1>faild</h1>")?;
                println!("faild");
            }
        }
        v => {
            let mut writer = BufWriter::new(&mut stream);
            writeln!(writer, "HTTP/1.1 200 OK\r")?;
            writeln!(writer, "Content-Type: text/html\r")?;
            writeln!(writer, "\r")?;
            writeln!(writer, "<h1>invalid id: '{}'</h1>", v)?;
        }
    };
    */
}

fn handle_home<R: io::Read + BufRead, W: io::Write>(
    reader: R,
    writer: W,
    method: Method,
) -> io::Result<Option<(R, W)>> {
    todo!()
}

fn handle_not_found<R: io::Read + BufRead, W: io::Write>(
    reader: R,
    writer: W,
    method: Method,
) -> io::Result<Option<(R, W)>> {
    todo!()
}

/*
    let client = ClientSync::<1024>::new();
    let mut sender = client.connect(KEY, addr)?;
    let mut output = Vec::new();
    // <id>\n<flow>
    sender.send(GetPwd::new(), &mut output, io::sink())?;
    sender.send(GetPwd::new(), &mut output, io::sink())?;
    sender.send(GetPwd::new(), &mut output, io::sink())?;
    sender.send(GetPwd::new(), &mut output, io::sink())?;
    sender.send(GetPwd::new(), &mut output, io::sink())?;
    let r = sender.send(GetPwd::new(), &mut output, io::sink())?;
    let v = unsafe { String::from_utf8_unchecked(output) };
    println!("{}", v);
    dbg!(r);
    sender.stop_server()?;
*/
