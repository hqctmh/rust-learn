mod error;
mod post;
mod user;

pub use self::error::Error;

pub type Result<T, E = Error> = ::std::result::Result<T, E>;
