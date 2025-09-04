use std::{
    io::{self, Read, Result, Write},
    sync::{Arc, Mutex, atomic::AtomicUsize},
};

use crate::app_data::AppData;

pub trait ReceiverApp {
    type Stream: Read + Write + Send + Sync;
    type BufStream: Read + Write + Send + Sync;
    type EStream: Read + Write + Send + Sync;
    type AppData: AppData + Send + Sync;
    type ConnectionHandler: Default + Send + Sync;
    fn connect(&self) -> impl FnMut() -> Option<Self::Stream>;
    fn is_accept_version(&self, version: (u32, u32, u32)) -> bool {
        self.get_version().0 == version.0
    }
    fn get_version(&self) -> (u32, u32, u32);
    fn to_buffer_stream(&self, stream: Self::Stream) -> Self::BufStream;
    fn handle_auth(&self, stream: &mut Self::BufStream) -> Result<bool>;
    fn encrypt_connection(
        &self,
        data: Arc<Mutex<Self::AppData>>,
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
    max_connection: usize,
    active_connection: AtomicUsize,
}

impl<App: ReceiverApp + Send + Sync + 'static> ReceiverAppServer<App> {
    pub fn run(self) -> io::Result<()> {
        let this = Arc::new(self);
        let mut connect_fn = this.app.connect();
        while let Some(stream) = connect_fn() {
            if this
                .active_connection
                .load(std::sync::atomic::Ordering::Relaxed)
                < this.max_connection
            {
                let this = this.clone();
                let s = std::thread::spawn(move || Self::handle_connection(this, stream));
            }
        }
        todo!()
    }
    fn handle_connection(this: Arc<Self>, stream: App::Stream) -> io::Result<()> {
        let mut buf_stream = this.app.to_buffer_stream(stream);
        let version = Self::get_sender_version(&mut buf_stream).unwrap();
        if this.app.is_accept_version(version) {}
        todo!()
    }
    fn get_sender_version(stream: &mut App::BufStream) -> io::Result<(u32, u32, u32)> {
        todo!()
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
            max_connection: 4,
            active_connection: AtomicUsize::new(0),
        }
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
    pub fn max_connection(mut self, v: usize) -> Self {
        self.max_connection = v;
        self
    }
}

#[cfg(test)]
mod tests {
    use crate::app::receiver_app::ReceiverAppServer;

    #[test]
    fn my_test() {
        //let server = ReceiverAppServer::new(|| );
    }
}
