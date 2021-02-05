//! This defines the GitHub RSS feed and API, to get the commits of the
//! repository (among other data).

use crate::prelude::*;
use once_cell::sync::Lazy;
use std::env;

pub static GITHUB: Lazy<Github> = Lazy::new(|| {
    let repository =
        env::var("GITHUB_REPOSITORY").unwrap_or_else(|_| String::from("PaperMC/Paper"));

    info!("Using Github repository: {}", repository);

    Github { repository }
});

static DOMAIN: Lazy<String> = Lazy::new(|| {
    env::var("GITHUB_DOMAIN").unwrap_or_else(|_| String::from("https://www.github.com"))
});
static API_DOMAIN: Lazy<String> = Lazy::new(|| {
    env::var("GITHUB_API_DOMAIN").unwrap_or_else(|_| String::from("https://api.github.com"))
});

pub struct Github {
    repository: String,
}

impl Github {}
