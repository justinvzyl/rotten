extern crate core;

mod action;
mod parser;
pub mod server;

/// Default port that the rotten server listens on.
pub const DEFAULT_PORT: u16 = 6969;

/// Error returned by most functions.
/// TODO: implement a error enum type
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// A convenience `Result` type
pub type Result<T> = std::result::Result<T, Error>;
