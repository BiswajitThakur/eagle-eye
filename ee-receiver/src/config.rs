/*
use crate::app::ReceiverConfigSync;

const BROADCAST_DATA_PREFIX: &str = ":eagle-eye:";

const MAX_CONNECTIONS: usize = 3;

pub fn config() -> ReceiverConfigSync {
    let key: [u8; 32] = [33; 32];
    let id: u128 = 123;

    ReceiverConfigSync::default()
        .broadcast_data_prefix(BROADCAST_DATA_PREFIX)
        .max_connection(MAX_CONNECTIONS)
        .key(key)
        .id(id)
}*/
