mod app;
mod config;
mod proto;
mod utils;

use std::{io, time::Duration};

use ee_task::prelude::*;

use crate::{app::AppSync, config::config, my_app::ThreadCounter};

fn main() -> io::Result<()> {
    let mut config = config();
    config
        .register::<RemoveFileSync>() // sender can remove file of receiver
        .register::<Ping>(); // sender can check, receiver is online or ofline.

    let app = AppSync::new(config);
    app.run()?;

    /*
    let v = ThreadCounter::new(5);
    v.span(|| {
        println!("thread started...");
        std::thread::sleep(Duration::from_secs(15));
        println!("thread closed...");
    });
    v.span(|| {
        println!("thread started...");
        std::thread::sleep(Duration::from_secs(15));
        println!("thread closed...");
    });
    v.span(|| {
        println!("thread started...");
        std::thread::sleep(Duration::from_secs(15));
        println!("thread closed...");
    });
    std::thread::sleep(Duration::from_secs(100));
    */
    /*

    let config = Config::new()
        .id(id)
        .password(passwd)
        .max_buf_size(64*1024)
        .max_broadcast_data_len(2*1024);

    let app = App::new(config);
    app.register::<RmFile>();
    app.register::<Ping>();

    proto::v2::run(app, |app: &App| {
        let a1 = app.exec(proto::recv_data::v1);
        let a2 = app.exec2(proto::proc_data::v1, a1);
        let a3 = app.exec2(proto::connect::v1, a2);
        let a4 = app.exec(proto::handle_conn::v1, a3);
        a4
    });

         * */
    Ok(())
}

pub mod my_app {
    use std::{
        io::{self},
        net::{SocketAddr, TcpStream},
        sync::{Arc, atomic::AtomicUsize},
        thread::JoinHandle,
    };

    use ee_broadcaster::ReceiverInfo;

    use crate::utils::process_broadcast_data;

    struct Config {
        id: u128,
        password: [u8; 32],
        max_broadcast_data_len: usize,
    }
    struct App {
        config: Config,
    }

    trait MyApp {
        fn get_id(&self) -> &u128;
        fn get_password(&self) -> &[u8; 32];
        fn get_prefix(&self) -> &str;
        fn get_max_broadcast_data_len(&self) -> usize;
        fn get_max_connection(&self) -> usize;
        fn get_handler(
            &self,
            value: &str,
        ) -> Option<impl FnOnce(Box<dyn io::Read>, Box<dyn io::Write>) -> io::Result<()>>;
    }

    fn run<T: MyApp>(app: T, f: impl FnOnce(TcpStream) -> io::Result<()>) -> io::Result<()> {
        let pass = *app.get_password();
        let id = *app.get_id();
        let counter = ThreadCounter::new(app.get_max_connection());
        let mut recv = ReceiverInfo::builder()
            .prefix(app.get_prefix())
            .buffer_size(8 * 1024)
            .socket_addr(SocketAddr::from(([255, 255, 255, 255], 6923)))
            .build()?;
        if let Ok(Some((add, buff))) = recv.next() {
            if let Some((addr, sec)) = process_broadcast_data(pass, id, add, buff) {};
        }
        Ok(())
    }

    pub struct ThreadCounter {
        max: usize,
        curr: Arc<AtomicUsize>,
    }

    impl ThreadCounter {
        pub fn new(v: usize) -> Self {
            Self {
                max: v,
                curr: Arc::new(AtomicUsize::new(0)),
            }
        }
        pub fn span<F, T>(&self, f: F) -> Option<JoinHandle<T>>
        where
            F: FnOnce() -> T,
            F: Send + 'static,
            T: Send + 'static,
        {
            if self.curr.load(std::sync::atomic::Ordering::SeqCst) < self.max {
                self.curr.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let count = self.curr.clone();
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
}
