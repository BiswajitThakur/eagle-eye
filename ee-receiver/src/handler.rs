use std::{net::TcpStream, sync::Arc};

use ee_stream::EStreamSync;
use ee_task::{GetId, file::RemoveFileSync};

use crate::receiver::EagleEyeServerSync;

pub fn handler<const N: usize>(
    key: [u8; 32],
) -> Arc<EagleEyeServerSync<EStreamSync<N, TcpStream, TcpStream>>> {
    let mut handler: EagleEyeServerSync<EStreamSync<N, TcpStream, TcpStream>> =
        EagleEyeServerSync::default().key(key);
    handler.register(RemoveFileSync::id(), RemoveFileSync::execute_on_server);
    Arc::new(handler)
}
