use std::{
    io,
    net::{SocketAddr, TcpListener},
};

use eagle_eye_jobs::RemoveFile;
use eagle_eye_proto::{
    server::EagleEyeServerSync,
    stream::EagleEyeStreamSync,
    task::{TaskRegisterySync, TaskSync},
};

//use eagle_eye_daemon::server::EagleEeDaemon;
fn main() -> io::Result<()> {
    let mut registery = TaskRegisterySync::new();
    registery.register(RemoveFile::_id(), RemoveFile::execute_on_sender);
    let listener = TcpListener::bind("127.69.69.69:6969".parse::<SocketAddr>().unwrap()).unwrap();
    let server: EagleEyeServerSync<EagleEyeStreamSync<1014, _, _>> =
        EagleEyeServerSync::new([1; 32], registery).set_log_path("error.log");
    server
        .run(
            listener
                .incoming()
                .map(|v| v.map(|m| (m.try_clone().unwrap(), m))),
        )
        .unwrap();
    Ok(())
}
