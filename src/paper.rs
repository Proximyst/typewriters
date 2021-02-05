//! This defines the PaperMC API, to get the builds and versions of the Paper
//! project.

use crate::prelude::*;
use once_cell::sync::Lazy;
use std::env;

pub static PAPER: Lazy<Paper> = Lazy::new(|| Paper { __priv: () });

static API_DOMAIN: Lazy<String> = Lazy::new(|| env::var("PAPER_API_DOMAIN").unwrap_or_else(|_| String::from("https://papermc.io/api")));

pub struct Paper {
    #[allow(dead_code)]
    __priv: (),
}

impl Paper {
}

