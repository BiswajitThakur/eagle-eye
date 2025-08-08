mod app;
mod config;
mod utils;

use std::io;

use ee_task::prelude::*;

use crate::{app::AppSync, config::config};

fn main() -> io::Result<()> {
    let mut buf = [0; 2048];

    let mut config = config::<512>().broadcast_buf(&mut buf);
    config.register::<RemoveFileSync>().register::<Ping>();

    let app = AppSync::new(config);
    app.run()?;

    Ok(())
}
