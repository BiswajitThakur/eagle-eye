use std::{
    io::{self, BufRead},
    net::{SocketAddr, TcpStream},
    ops::{Index, IndexMut},
};

use crate::{
    stream::EagleEyeStreamSync,
    utils::{handle_stream_client_sync, handle_stream_server_sync},
};

#[derive(Clone)]
pub struct JobSync<const N: usize> {
    id: &'static str,
    server: fn(&mut EagleEyeStreamSync<N, &TcpStream, &TcpStream>) -> io::Result<Connection>,
    client: fn(&mut EagleEyeStreamSync<N, &TcpStream, &TcpStream>) -> io::Result<Connection>,
}

pub trait ClientJobSync {
    fn id(&self) -> &'static str;
    fn send(&self) -> Self;
}

#[derive(Clone, Copy)]
pub enum Connection {
    Close,
    Continue,
}

#[derive(Clone)]
pub struct JobsSync<const N: usize>(Vec<JobSync<N>>);

impl<const N: usize> Index<usize> for JobsSync<N> {
    type Output = JobSync<N>;
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<const N: usize> IndexMut<usize> for JobsSync<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<const N: usize> JobsSync<N> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn insert(
        &mut self,
        id: &'static str,
        server: fn(&mut EagleEyeStreamSync<N, &TcpStream, &TcpStream>) -> io::Result<Connection>,
        client: fn(&mut EagleEyeStreamSync<N, &TcpStream, &TcpStream>) -> io::Result<Connection>,
    ) {
        let job = JobSync { id, server, client };
        self.0.push(job);
        self.0.sort_by(|a, b| a.id.cmp(b.id));
    }
    pub fn find_server(
        &self,
        id: &str,
    ) -> Option<&fn(&mut EagleEyeStreamSync<N, &TcpStream, &TcpStream>) -> io::Result<Connection>>
    {
        self.0
            .binary_search_by(|v| v.id.cmp(id))
            .ok()
            .map(|v| &self[v].server)
    }
    pub fn find_client(
        &self,
        id: &str,
    ) -> Option<&fn(&mut EagleEyeStreamSync<N, &TcpStream, &TcpStream>) -> io::Result<Connection>>
    {
        self.0
            .binary_search_by(|v| v.id.cmp(id))
            .ok()
            .map(|v| &self[v].client)
    }
}

impl<const N: usize> JobsSync<N> {
    pub fn run_server(&self, key: [u8; 32], stream: TcpStream) -> io::Result<()> {
        let mut e_stream = handle_stream_server_sync::<N>(key, &stream)
            .unwrap()
            .unwrap();
        loop {
            let job_id = (&mut e_stream).lines().next().unwrap().unwrap();
            let f = self.find_server(&job_id).unwrap();
            match f(&mut e_stream) {
                Ok(Connection::Continue) => continue,
                Ok(Connection::Close) => break,
                Err(err) => panic!("{}", err),
            };
        }
        Ok(())
    }
}

pub struct JobSenderSync<T> {
    stream: T,
}

impl<const N: usize> JobSenderSync<N> {
    pub fn send() {}
}

#[cfg(test)]
mod tests {
    use std::{io, net::TcpStream};

    use crate::stream::EagleEyeStreamSync;

    use super::{Connection, JobsSync};

    fn v1<const N: usize>(
        _: &mut EagleEyeStreamSync<N, &TcpStream, &TcpStream>,
    ) -> io::Result<Connection> {
        todo!()
    }
    fn v2<const N: usize>(
        _: &mut EagleEyeStreamSync<N, &TcpStream, &TcpStream>,
    ) -> io::Result<Connection> {
        todo!()
    }
    #[test]
    fn tt() {
        let mut jobs = JobsSync::<1>::new();
        jobs.insert("c", v2, v1);
        jobs.insert("b", v1, v2);
        jobs.insert("a", v1, v1);
        jobs.insert("e", v2, v2);
        jobs.insert("d", v2, v2);
        assert!(jobs.find_server("d").is_some());
    }
}
