use std::io;

pub trait App {
    fn from<T: Config>(value: T) -> Self;
    fn key(&self) -> &[u8; 32];
    fn run(self) -> io::Result<()>;
}

pub trait Config {
    fn into<T: App>(self) -> T;
}
