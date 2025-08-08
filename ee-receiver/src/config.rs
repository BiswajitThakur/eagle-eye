use std::net::TcpStream;

use ee_stream::EStreamSync;

use crate::receiver::ReceiverConfigSync;

pub fn config<'a, const N: usize>() -> ReceiverConfigSync<'a, EStreamSync<N, TcpStream, TcpStream>>
{
    
    ReceiverConfigSync::default()
}
