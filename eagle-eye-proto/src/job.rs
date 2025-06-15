use std::io;

struct JobSync<T: io::Read + io::Write> {
    id: &'static str,
    server: fn(T) -> io::Result<T>,
    client: fn(T) -> io::Result<T>,
}

pub struct JobsSync<T: io::Read + io::Write>(Vec<JobSync<T>>);

impl<T: io::Read + io::Write> JobsSync<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn insert(
        &mut self,
        id: &'static str,
        server: fn(T) -> io::Result<T>,
        client: fn(T) -> io::Result<T>,
    ) {
        let job = JobSync { id, server, client };
        self.0.push(job);
        self.0.sort_by(|a, b| a.id.cmp(b.id));
    }
    pub fn find_server(&self, id: &str) -> Option<&fn(T) -> io::Result<T>> {
        self.0
            .binary_search_by(|v| v.id.cmp(id))
            .ok()
            .map(|v| &self.0[v].server)
    }
    pub fn find_client(&self, id: &str) -> Option<&fn(T) -> io::Result<T>> {
        self.0
            .binary_search_by(|v| v.id.cmp(id))
            .ok()
            .map(|v| &self.0[v].client)
    }
}

#[cfg(test)]
mod tests {
    use std::{io, net::TcpStream};

    use super::JobsSync;

    fn v1<T: io::Read + io::Write>(_: T) -> io::Result<T> {
        todo!()
    }
    fn v2<T: io::Read + io::Write>(_: T) -> io::Result<T> {
        todo!()
    }
    #[test]
    fn tt() {
        let mut jobs = JobsSync::<TcpStream>::new();
        jobs.insert("c", v2, v1);
        jobs.insert("b", v1, v2);
        jobs.insert("a", v1, v1);
        jobs.insert("e", v2, v2);
        jobs.insert("d", v2, v2);
        assert!(jobs.find_server("d").is_some());
    }
}
