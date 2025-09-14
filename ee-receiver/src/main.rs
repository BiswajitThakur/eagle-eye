mod app;
mod config;
mod data;
mod handler;
mod utils;

use std::io;

use ee_app::receiver::sync::server::Server;

use crate::{app::App, data::AppData, handler::ConnectionHandler, utils::auth};

fn main() -> io::Result<()> {
    let mut server = Server::new(move || App::new());

    server.app_name("eagle-eye");
    server.version((1, 0, 0));
    server.handler(ConnectionHandler::default());

    server.max_connection(8);
    server.auth(auth);
    server.app_data(AppData::new());

    server.run();
    Ok(())
}
