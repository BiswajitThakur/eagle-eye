use std::{
    io,
    net::{SocketAddr, TcpListener},
};

use eagle_eye_jobs::RemoveFile;
use eagle_eye_proto::{server::EagleEyeServerSync, stream::EagleEyeStreamSync, task::GetId};

//use eagle_eye_daemon::server::EagleEeDaemon;
fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.69.69.69:6969".parse::<SocketAddr>().unwrap()).unwrap();

    let mut server: EagleEyeServerSync<EagleEyeStreamSync<1014, _, _>> =
        EagleEyeServerSync::default();

    server.register(RemoveFile::id(), RemoveFile::execute_on_server);

    server
        .run(
            listener
                .incoming()
                .map(|v| v.map(|m| (m.try_clone().unwrap(), m))),
        )
        .unwrap();
    Ok(())
}
