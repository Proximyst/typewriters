#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]
#![warn(clippy::all)]

pub mod prelude {
    pub use tracing::{debug, error, info, trace, warn};
}

pub static REQWEST: Lazy<ReqwestClient> = Lazy::new(|| {
    ReqwestClient::builder()
        .user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
            " automated-bot (+https://github.com/Proximyst/typewriters.git)"
        ))
        .build()
        .wrap_err("cannot create reqwest client")
        .unwrap()
});

use self::prelude::*;
use once_cell::sync::Lazy;
use reqwest::Client as ReqwestClient;
use stable_eyre::eyre::{Report, WrapErr as _};

#[tokio::main]
async fn main() -> Result<(), Report> {
    // We first need Eyre to work correctly...
    stable_eyre::install()?;

    // Now to init the .env file...
    match dotenv::dotenv() {
        Ok(_) => (),
        Err(e) if e.not_found() => (),
        Err(e) => {
            return Err(e).wrap_err(".env file could not be loaded");
        }
    }

    // And now tracing, as that depends on .env...
    tracing_subscriber::fmt::init();

    Ok(())
}
