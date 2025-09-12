use std::{
    error::Error,
    io::{self, Read, Result, Write},
    num::NonZeroUsize,
    sync::{Arc, Mutex, atomic::AtomicUsize},
    thread::JoinHandle,
};

use crate::app_data::AppData;

pub trait ReceiverApp {
    type Stream: Read + Write + Send + Sync;
    type BufStream: Read + Write + Send + Sync;
    type EStream: Read + Write + Send + Sync;
    type AppData: Default + Send + Sync;
    type ConnectionHandler: Default + Send + Sync;
    fn get_stream(&self) -> impl FnMut() -> Option<Self::Stream>;
    fn accept_version(&self) -> impl Fn((u32, u32, u32)) -> bool;
    fn to_buffer_stream(&self, stream: Self::Stream) -> Self::BufStream;
    fn handle_auth(&self, stream: &mut Self::BufStream) -> Result<bool>;
    fn log_error<E: Error>(&self, _error: E) {}
    fn encrypt_connection(
        &self,
        data: &Arc<Mutex<Self::AppData>>,
        stream: Self::BufStream,
    ) -> Result<Self::EStream>;
    fn handle_connection(
        data: Arc<Mutex<Self::AppData>>,
        handler: Arc<Self::ConnectionHandler>,
        stream: Self::EStream,
    ) -> io::Result<()>;
}

pub struct ReceiverAppServer<App>
where
    App: ReceiverApp + Send + Sync + 'static,
{
    version: (u32, u32, u32),
    app_name: &'static str,
    app: Arc<App>,
    app_data: Arc<Mutex<App::AppData>>,
    connection_handler: Arc<App::ConnectionHandler>,
    auth: Box<dyn Fn(Arc<App>, &mut App::BufStream) -> io::Result<bool>>,
    max_connection: NonZeroUsize,
    active_connection: Arc<AtomicUsize>,
}

impl<App: ReceiverApp + Send + Sync + 'static> ReceiverAppServer<App> {
    pub fn run(self) {
        let is_connect = self.app.accept_version();
        let mut get_stream = self.app.get_stream();
        while let Some(stream) = get_stream() {
            let data = self.app_data.clone();
            let handler = self.connection_handler.clone();
            let mut stream = App::to_buffer_stream(&self.app, stream);
            let r = self.connect(&mut stream, &is_connect);
            if let Err(err) = r {
                App::log_error(&self.app, err);
                continue;
            } else {
                if !r.unwrap() {
                    continue;
                }
            }
            let r = App::handle_auth(&self.app, &mut stream);
            if let Err(err) = r {
                App::log_error(&self.app, err);
                continue;
            } else {
                if !r.unwrap() {
                    continue;
                }
            }
            let e_stream = App::encrypt_connection(&self.app, &data, stream);
            if let Err(ref err) = e_stream {
                self.app.log_error(err);
            }
            self.span(|| App::handle_connection(data, handler, e_stream.unwrap()))
                .unwrap();
        }
    }
    fn connect(
        &self,
        stream: &mut App::BufStream,
        f: impl Fn((u32, u32, u32)) -> bool,
    ) -> io::Result<bool> {
        // sender send
        // <app-name><version>
        let mut buf = [0u8; 4];
        let buf_len = buf.len();
        let mut app_name = self.app_name.as_bytes();
        // read prefix (app name)
        loop {
            if app_name.is_empty() {
                break;
            }
            let n = stream.read(&mut buf[0..std::cmp::min(app_name.len(), buf_len)])?;
            if &buf[0..n] != &app_name[0..n] {
                return Ok(false);
            }
            app_name = &app_name[n..];
        }
        // read version
        stream.read_exact(&mut buf)?;
        let major = u32::from_be_bytes(buf);
        stream.read_exact(&mut buf)?;
        let minor = u32::from_be_bytes(buf);
        stream.read_exact(&mut buf)?;
        let patch = u32::from_be_bytes(buf);
        let version = (major, minor, patch);
        if f(version) {
            stream.write_all(b":ok:")?;
            Ok(true)
        } else {
            stream.write_all(b":version_not_accepted:")?;
            Ok(false)
        }
    }
    pub fn new<F: FnOnce() -> App + Send + Sync + 'static>(f: F) -> Self {
        let app = f();
        let data = Arc::new(Mutex::new(App::AppData::default()));
        let handler = Arc::new(App::ConnectionHandler::default());
        Self {
            version: (0, 0, 0),
            app_name: "eagle-eye",
            app: Arc::new(app),
            app_data: data,
            connection_handler: handler,
            auth: Box::new(|_, _| Ok(true)),
            max_connection: unsafe { NonZeroUsize::new_unchecked(4) },
            active_connection: Arc::new(AtomicUsize::new(0)),
        }
    }
    pub fn auth(
        mut self,
        f: impl Fn(Arc<App>, &mut App::BufStream) -> io::Result<bool> + 'static,
    ) -> Self {
        self.auth = Box::new(f);
        self
    }
    pub fn version(mut self, version: (u32, u32, u32)) -> Self {
        self.version = version;
        self
    }
    pub fn app_name(mut self, name: &'static str) -> Self {
        self.app_name = name;
        self
    }
    pub fn app(mut self, app: App) -> Self {
        self.app = Arc::new(app);
        self
    }
    pub fn app_data(mut self, data: App::AppData) -> Self {
        self.app_data = Arc::new(Mutex::new(data));
        self
    }
    pub fn handler(mut self, handler: App::ConnectionHandler) -> Self {
        self.connection_handler = Arc::new(handler);
        self
    }
    pub fn max_connection(mut self, n: usize) -> Self {
        self.max_connection = NonZeroUsize::new(n).expect("max connection can not be zero");
        self
    }
    fn span<F, T>(&self, f: F) -> Option<JoinHandle<T>>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        if self
            .active_connection
            .load(std::sync::atomic::Ordering::SeqCst)
            < self.max_connection.get()
        {
            self.active_connection
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            let count = self.active_connection.clone();
            Some(std::thread::spawn(move || {
                let v = f();
                count.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
                v
            }))
        } else {
            None
        }
    }
}
