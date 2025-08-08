mod config;
mod receiver;
mod utils;

use std::io;

use ee_task::prelude::*;

use crate::{config::config, receiver::AppSync};

const BROADCAST_DATA_PREFIX: &str = ":eagle-eye:";

const MAX_CONNECTIONS: usize = 3;

fn main() -> io::Result<()> {
    let key: [u8; 32] = [33; 32];
    let id: u128 = 123;
    let mut broad_buf = [0; 2048];

    let mut config = config::<512>()
        .broadcast_buf(&mut broad_buf)
        .broadcast_data_prefix(BROADCAST_DATA_PREFIX)
        .max_connection(MAX_CONNECTIONS)
        .key(key)
        .id(id);

    config.register::<RemoveFileSync>();
    config.register::<Ping>();

    let app = AppSync::new(config);
    app.run()?;

    Ok(())
}
