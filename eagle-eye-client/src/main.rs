use std::{
    io::{self, BufRead, BufReader, BufWriter, Write},
    net::TcpListener,
};

use eagle_eye_jobs::RemoveFile;
use eagle_eye_proto::{client::ClientSync, server};

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7575")?;
    for stream in listener.incoming() {
        let stream = stream?;
        handle_stream(stream)?;
    }
    Ok(())
}

fn handle_stream<T: io::Read + io::Write>(mut stream: T) -> io::Result<()> {
    let client = ClientSync::<1024>::new();
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
        v if v == RemoveFile::_id() => {
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
    Ok(())
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
