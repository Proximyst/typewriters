#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]
#![warn(clippy::all)]

pub mod prelude {
    pub use crate::stream::*;
    pub use async_trait::async_trait;
    pub use getset::*;
    pub use snafu::{ResultExt, Snafu};
    pub use tracing::{debug, error, info, trace, warn};

    use crate::{github::Github, paper::PaperApi};
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

    // TODO: Find a better place for these?
    pub static GITHUB: Lazy<Github> = Lazy::new(|| {
        let repository = env::var("GITHUB_REPOSITORY").unwrap_or_else(|_| String::from("PaperMC/Paper"));

        info!("Using Github repository: {}", repository);

        Github::new(repository)
    });

    static GITHUB_DOMAIN: Lazy<String> =
        Lazy::new(|| env::var("GITHUB_DOMAIN").unwrap_or_else(|_| String::from("https://www.github.com")));
    static GITHUB_API_DOMAIN: Lazy<String> = Lazy::new(|| {
        env::var("GITHUB_API_DOMAIN").unwrap_or_else(|_| String::from("https://api.github.com"))
    });

    static PAPER_API_DOMAIN: Lazy<String> =
        Lazy::new(|| env::var("PAPER_API_DOMAIN").unwrap_or_else(|_| String::from("https://papermc.io/api")));

    static PAPER_PROJECT_NAME: Lazy<String> =
        Lazy::new(|| env::var("PAPER_PROJECT").unwrap_or_else(|_| String::from("paper")));

    pub static PAPER: Lazy<PaperApi> = Lazy::new(|| PaperApi::new(&*PAPER_API_DOMAIN, &*PAPER_PROJECT_NAME));
}

mod data;
mod github;
mod paper;
mod stream;

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
