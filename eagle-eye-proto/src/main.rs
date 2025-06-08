use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    time::Duration,
};

use eagle_eye_proto::{handle_stream_client_sync, handle_stream_server_sync};

fn main() -> io::Result<()> {
    let server = TcpListener::bind("127.69.69.69:6969")?;
    let t = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(1));
        let stream = TcpStream::connect("127.69.69.69:6969").unwrap();
        let key = [10; 32];
        let mut stream = match handle_stream_client_sync::<512>(key, &stream).unwrap() {
            Some(v) => v,
            None => {
                eprintln!("Wrong Password");
                return;
            }
        };
        stream.write_all(b"-----\n").unwrap();
        stream.write_all(b"hello\n").unwrap();
        stream.write_all(b"world\n").unwrap();
        stream.write_all(b"-----\n").unwrap();
    });
    let mut stdout = std::io::stdout();
    let mut buf = [0; 100];
    for stream in server.incoming() {
        let stream = stream?;

        let key = [10; 32];
        let mut stream = match handle_stream_server_sync::<1024>(key, &stream).unwrap() {
            Some(v) => v,
            _ => {
                eprintln!("Wrong Password...");
                break;
            }
        };
        loop {
            let n = stream.read(&mut buf)?;
            if n == 0 {
                break;
            }
            stdout.write_all(&buf[0..n])?;
            stdout.flush()?;
        }
        break;
    }
    t.join().unwrap();
    Ok(())
}
