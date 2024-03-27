use lazy_static::lazy_static;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};

pub mod messages;

// This is the connection to the database for the entire program.
// There should never be a new connection made, this will cause the information
// from the past connection to no longer persist.
lazy_static! {
    pub static ref CONN: Arc<Mutex<Connection>> =
    Arc::new(
    Mutex::new(
    Connection::open_in_memory()
    .expect("Could not create Connection.")));
}