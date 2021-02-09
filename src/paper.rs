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
#[cfg(test)]
use mockito;

pub static PAPER: Lazy<Paper> = Lazy::new(|| Paper { __priv: () });

#[cfg(not(test))]
static API_DOMAIN: Lazy<String> = Lazy::new(|| {
    env::var("PAPER_API_DOMAIN").unwrap_or_else(|_| String::from("https://papermc.io/api"))
});
#[cfg(test)]
static API_DOMAIN: Lazy<String> = Lazy::new(|| String::from(&mockito::server_url()));

#[cfg(not(test))]
static PROJECT_NAME: Lazy<String> =
    Lazy::new(|| env::var("PAPER_PROJECT").unwrap_or_else(|_| String::from("paper")));

#[cfg(test)]
static PROJECT_NAME: Lazy<String> = Lazy::new(|| String::from("paper"));

const ACCEPT_JSON: &'static str = "application/json";

pub struct Paper {
    #[allow(dead_code)]
    __priv: (),
}

#[derive(Deserialize, Getters, Debug, PartialEq)]
#[getset(get = "pub")]
pub struct Project {
    version_groups: Vec<String>,
    versions: Vec<String>,
}

#[derive(Deserialize, Getters, Debug, PartialEq)]
#[getset(get = "pub")]
pub struct Version {
    builds: Vec<i32>,
}

#[derive(Deserialize, Getters, CopyGetters, Debug, PartialEq)]
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

#[derive(Deserialize, Getters, Debug, PartialEq)]
#[getset(get = "pub")]
pub struct Change {
    commit: String,
    summary: String,
    message: String,
}

#[derive(Deserialize, Getters, Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::{assert_eq, assert_ne};
    use mockito::mock;
    use tokio;
    use maplit::hashmap;

    #[tokio::test]
    async fn check_project_parsing() {
        let project_mock = mock("GET", "/v2/projects/paper")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
              "project_id": "paper",
              "project_name": "Paper",
              "version_groups": [
                "1.16",
                "1.17"
              ],
              "versions": [
                "1.16.3",
                "1.16.4",
                "1.16.5",
                "1.17.0"
              ]
            }"#)
            .create();
        let expected = Project {
            version_groups: vec!["1.16", "1.17"].iter().map(|&s| s.into()).collect(),
            versions: vec!["1.16.3", "1.16.4", "1.16.5", "1.17.0"].iter().map(|&s| s.into()).collect(),
        };
        let actual = PAPER.get_project().await.expect("Error getting project");
        project_mock.assert();
        assert_eq!(actual, expected);
    }


    #[tokio::test]
    async fn check_version_parsing() {
        let project_mock = mock("GET", "/v2/projects/paper/versions/1.16.5")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
              "project_id": "paper",
              "project_name": "Paper",
              "version": "1.16.5",
              "builds": [
                463,
                464,
                465,
                466
              ]
            }"#)
            .create();
        let expected = Version { builds: vec![463, 464, 465, 466] };
        let actual = PAPER.get_version("1.16.5").await.expect("Error getting version");
        project_mock.assert();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn check_build_parsing() {
        let project_mock = mock("GET", "/v2/projects/paper/versions/1.16.5/builds/466")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
              "project_id": "paper",
              "project_name": "Paper",
              "version": "1.16.5",
              "build": 466,
              "time": "2021-02-08T10:22:13.662Z",
              "changes": [
                {
                  "commit": "36a72cad3098a513375068008d3720d3aebc2d82",
                  "summary": "ChangeSummary",
                  "message": "ChangeMessage"
                }
              ],
              "downloads": {
                "application": {
                  "name": "paper-1.16.5-466.jar",
                  "sha256": "58275a88331dc21c857be49fd7a9d70ba04843253e73e8a7424160b34529e04a"
                }
              }
            }"#)
            .create();
        let expected = VersionBuild {
            build: 466,
            time: Utc.ymd(2021,02,08).and_hms_milli(10, 22, 13, 662),
            changes: vec![Change {
                commit: "36a72cad3098a513375068008d3720d3aebc2d82".into(),
                summary: "ChangeSummary".into(),
                message: "ChangeMessage".into(),
            }],
            downloads: hashmap! {
                "application".into() => Download {
                    name: "paper-1.16.5-466.jar".into(),
                    sha256: "58275a88331dc21c857be49fd7a9d70ba04843253e73e8a7424160b34529e04a".into(),
                }
            },
        };
        let actual = PAPER.get_build("1.16.5", 466).await.expect("Error getting build");
        project_mock.assert();
        assert_eq!(actual, expected);
    }
}
