pub mod cli;
pub mod client;
pub mod error;
pub mod mcp;
#[doc(hidden)]
pub mod test_support;
pub mod types;
pub mod util;

mod codec;

pub use client::{EdcbClient, PluginKind, build_reservation_from_event};
pub use error::{EdcbError, Result};
pub use types::{EventKey, ProgramSearchQuery, ServiceKey};
