use std::{net::TcpStream, sync::Arc};

use eagle_eye_proto::{server::EagleEyeServerSync, task::GetId};

use eagle_eye_jobs::file::RemoveFile;
use ee_stream::EStreamSync;

pub fn handler<const N: usize>(
    key: [u8; 32],
) -> Arc<EagleEyeServerSync<EStreamSync<N, TcpStream, TcpStream>>> {
    let mut handler: EagleEyeServerSync<EStreamSync<N, TcpStream, TcpStream>> =
        EagleEyeServerSync::default().key(key);
    handler.register(RemoveFile::id(), RemoveFile::execute_on_server);
    Arc::new(handler)
}
