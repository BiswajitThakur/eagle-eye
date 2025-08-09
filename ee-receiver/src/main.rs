mod app;
mod config;
mod utils;

use std::io;

use ee_task::prelude::*;

use crate::{app::AppSync, config::config};

fn main() -> io::Result<()> {
    let mut buf = [0; 2048];

    let mut config = config::<512>().broadcast_buf(&mut buf);
    config
        .register::<RemoveFileSync>() // sender can remove file of receiver
        .register::<Ping>(); // sender can check, receiver is online or ofline.

    let app = AppSync::new(config);
    app.run()?;

    /*

    let config = Config::new()
        .id(id)
        .password(passwd)
        .max_buf_size(64*1024)
        .max_broadcast_data_len(2*1024);

    let app = App::new(config);
    app.register::<RmFile>();
    app.register::<Ping>();

    proto::v2::run(app);

         * */
    Ok(())
}
