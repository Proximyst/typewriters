//! This defines the PaperMC API, to get the builds and versions of the Paper
//! project.

use crate::prelude::*;
use chrono::prelude::*;
use once_cell::sync::Lazy;
use reqwest::header;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use url::Url;

pub static PAPER: Lazy<Paper> = Lazy::new(|| Paper { __priv: () });

static API_DOMAIN: Lazy<String> = Lazy::new(|| {
    env::var("PAPER_API_DOMAIN").unwrap_or_else(|_| String::from("https://papermc.io/api"))
});
static PROJECT_NAME: Lazy<String> =
    Lazy::new(|| env::var("PAPER_PROJECT").unwrap_or_else(|_| String::from("paper")));

const ACCEPT_JSON: &'static str = "application/json";

pub struct Paper {
    #[allow(dead_code)]
    __priv: (),
}

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct Project {
    version_groups: Vec<String>,
    versions: Vec<String>,
}

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct Version {
    builds: Vec<i32>,
}

#[derive(Deserialize, Getters, CopyGetters)]
pub struct VersionBuild {
    #[getset(get_copy = "pub")]
    build: i32,
    #[getset(get_copy = "pub")]
    time: DateTime<Utc>,
    #[getset(get = "pub")]
    changes: Vec<Change>,
    #[getset(get = "pub")]
    downloads: HashMap<String, Download>,
}

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct Change {
    commit: String,
    summary: String,
    message: String,
}

#[derive(Deserialize, Getters)]
#[getset(get = "pub")]
pub struct Download {
    name: String,
    sha256: String,
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("cannot parse url \"{}\": {}", url, source))]
    UrlParse {
        source: url::ParseError,
        url: String,
    },

    #[snafu(display("cannot fetch project: {}", source))]
    ProjectFetch { source: reqwest::Error },

    #[snafu(display("cannot fetch version {}: {}", version, source))]
    VersionFetch {
        source: reqwest::Error,
        version: String,
    },

    #[snafu(display("cannot fetch build {} b{}: {}", version, build, source))]
    BuildFetch {
        source: reqwest::Error,
        version: String,
        build: i32,
    },

    #[snafu(display("invalid body received: {}", source))]
    InvalidBody { source: reqwest::Error },

    #[snafu(display("invalid json returned: {}\nbody: {}", source, body))]
    InvalidJson {
        source: serde_json::Error,
        body: String,
    },
}

type Result<T, E = Error> = std::result::Result<T, E>;

impl Paper {
    pub async fn get_project(&self) -> Result<Project> {
        let url = format!("{}/v2/projects/{}", &*API_DOMAIN, &*PROJECT_NAME);
        let url = Url::parse(&url).context(UrlParse { url })?;

        let response = REQWEST
            .get(url)
            .header(header::ACCEPT, ACCEPT_JSON)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .context(ProjectFetch)?;

        let body = response.text().await.context(InvalidBody)?;

        serde_json::from_str(&body).context(InvalidJson { body })
    }

    pub async fn get_version(&self, version: &str) -> Result<Version> {
        let url = format!(
            "{}/v2/projects/{}/versions/{}",
            &*API_DOMAIN, &*PROJECT_NAME, version,
        );
        let url = Url::parse(&url).context(UrlParse { url })?;

        let response = REQWEST
            .get(url)
            .header(header::ACCEPT, ACCEPT_JSON)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .context(VersionFetch { version })?;

        let body = response.text().await.context(InvalidBody)?;

        serde_json::from_str(&body).context(InvalidJson { body })
    }

    pub async fn get_build(&self, version: &str, build: i32) -> Result<VersionBuild> {
        let url = format!(
            "{}/v2/projects/{}/versions/{}/builds/{}",
            &*API_DOMAIN, &*PROJECT_NAME, version, build,
        );
        let url = Url::parse(&url).context(UrlParse { url })?;

        let response = REQWEST
            .get(url)
            .header(header::ACCEPT, ACCEPT_JSON)
            .send()
            .await
            .and_then(|r| r.error_for_status())
            .context(BuildFetch { version, build })?;

        let body = response.text().await.context(InvalidBody)?;

        serde_json::from_str(&body).context(InvalidJson { body })
    }
}
