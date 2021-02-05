#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]
#![warn(clippy::all)]

pub mod prelude {
    pub use tracing::{debug, error, info, trace, warn};

    use once_cell::sync::Lazy;
    use reqwest::Client as ReqwestClient;
    use stable_eyre::eyre::WrapErr as _;
    use std::env;

    pub static REQWEST: Lazy<ReqwestClient> = Lazy::new(|| {
        ReqwestClient::builder()
            .user_agent(env::var("USER_AGENT").unwrap_or_else(|_| {
                String::from(concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION"),
                    " automated-bot (+https://github.com/Proximyst/typewriters.git)"
                ))
            }))
            .build()
            .wrap_err("cannot create reqwest client")
            .unwrap()
    });
}

mod data;
mod github;
mod paper;

use self::prelude::*;
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
