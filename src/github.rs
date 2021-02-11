//! This defines the GitHub RSS feed and API, to get the commits of the
//! repository (among other data).

use crate::prelude::*;

const ACCEPT_ATOM: &'static str = "application/atom+xml";


pub struct Github {
    repository: String,
}


impl Default for Github {
    fn default() -> Self {
        Self::new("PaperMC/Paper")
    }
}

impl Github {
    pub fn new<T>(repository: T) -> Self where T: Into<String> {
        Self { repository: repository.into() }
    }
}
