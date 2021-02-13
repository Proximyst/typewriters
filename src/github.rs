//! This defines the GitHub RSS feed and API, to get the commits of the
//! repository (among other data).

const ACCEPT_ATOM: &'static str = "application/atom+xml";

pub struct Github {
    repository: String,
    domain: String,
    api_domain: String,
}

impl Github {
    pub fn new(
        repository: impl Into<String>,
        domain: impl Into<String>,
        api_domain: impl Into<String>,
    ) -> Self {
        Self {
            repository: repository.into(),
            domain: domain.into(),
            api_domain: api_domain.into(),
        }
    }
}
