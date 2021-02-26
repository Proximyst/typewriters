//! This defines the PaperMC API, to get the builds and versions of the Paper
//! project.

use crate::prelude::*;
use chrono::prelude::*;
#[cfg(test)]
use mockito;
use reqwest::header;
use serde::Deserialize;
use snafu::OptionExt;
use std::collections::HashMap;

use url::Url;

const ACCEPT_JSON: &'static str = "application/json";

#[derive(Getters)]
#[getset(get = "pub", set = "pub")]
pub struct PaperApi {
    domain: String,
    project: String,
}

#[derive(Deserialize, Getters, Debug, PartialEq)]
#[getset(get = "pub")]
pub struct Project {
    version_groups: Vec<String>,
    versions: Vec<String>,
}

/// Type of the number representing a build
// i32 because Bibliothek is Java API using signed integers
type BuildNumber = i32;
#[derive(Deserialize, Getters, Debug, PartialEq)]
#[getset(get = "pub")]
pub struct Version {
    builds: Vec<BuildNumber>,
}

impl Version {
    pub fn get_latest_build_number(&self) -> Option<BuildNumber> {
        self
            .builds
            .last()
            .copied()
    }
}

#[derive(Deserialize, Getters, CopyGetters, Debug, PartialEq)]
pub struct VersionBuild {
    #[getset(get_copy = "pub")]
    build: BuildNumber,
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
    UrlParse { source: url::ParseError, url: String },

    #[snafu(display("cannot fetch project: {}", source))]
    ProjectFetch { source: reqwest::Error },

    #[snafu(display("cannot fetch version {}: {}", version, source))]
    VersionFetch { source: reqwest::Error, version: String },

    #[snafu(display("cannot fetch build {} b{}: {}", version, build, source))]
    BuildFetch {
        source: reqwest::Error,
        version: String,
        build: i32,
    },

    #[snafu(display("invalid body received: {}", source))]
    InvalidBody { source: reqwest::Error },

    #[snafu(display("invalid json returned: {}\nbody: {}", source, body))]
    InvalidJson { source: serde_json::Error, body: String },

    #[snafu(display("no builds for version {:?}", version))]
    NoBuilds { version: Version },
}

type Result<T, E = Error> = std::result::Result<T, E>;

impl PaperApi {
    /// Create new PaperApi, using given domain and project
    pub fn new(domain: impl Into<String>, project: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            project: project.into(),
        }
    }

    pub async fn get_project(&self) -> Result<Project> {
        let url = format!("{}/v2/projects/{}", self.domain, self.project);
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
            self.domain, self.project, version,
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
            self.domain, self.project, version, build,
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

    pub async fn get_latest_build(&self, version: &str) -> Result<VersionBuild> {
        let paper_version = self.get_version(version).await?;
        self.get_build(version, paper_version.get_latest_build_number().context(NoBuilds { version: paper_version })?)
            .await
    }
}

pub struct PaperStream {
    last_build: Option<BuildNumber>,
    api: PaperApi,
    version: String,
}

impl PaperStream {
    pub fn new<T>(api: PaperApi, version: T) -> Self
    where
        T: Into<String>,
    {
        PaperStream {
            last_build: None,
            api,
            version: version.into(),
        }
    }
}

#[async_trait]
impl UpdateStream for PaperStream {
    // TODO: Change to VersionBuild?
    type Item = Version;
    type Error = Error;

    async fn fetch_update(&mut self) -> UpdateResult<Self::Item, Self::Error> {
        let version_info = self.api.get_version(self.version.as_str()).await?;
        let latest_build = version_info.get_latest_build_number();
        match self.last_build {
            // TODO: Decide whether first run should always return an update or not
            None => {
                self.last_build = latest_build;
                Ok(None)
            }
            Some(build) => {
                match latest_build {
                    Some(latest_build_number) => {
                        if (latest_build_number > build) {
                            // Got a new version!
                            self.last_build = Some(latest_build_number);
                            Ok(Some(version_info))
                        } else if (latest_build_number == build) {
                            // Nothing changed
                            Ok(None)
                        } else {
                            // TODO: We should probably handle it in case of hitting a different cached endpoint
                            panic!("Did we go back in time?");
                        }
                    },
                    None => NoBuilds { version: version_info }.fail()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::hashmap;
    use mockito::mock;
    use pretty_assertions::assert_eq;
    use tokio;

    fn get_paper_api() -> PaperApi {
        let domain = &mockito::server_url();
        let project = "paper";
        PaperApi::new(domain, project)
    }

    #[tokio::test]
    async fn check_project_parsing() {
        let project_mock = mock("GET", "/v2/projects/paper")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
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
            }"#,
            )
            .create();
        let expected = Project {
            version_groups: vec!["1.16", "1.17"].iter().map(|&s| s.into()).collect(),
            versions: vec!["1.16.3", "1.16.4", "1.16.5", "1.17.0"]
                .iter()
                .map(|&s| s.into())
                .collect(),
        };
        let actual = get_paper_api()
            .get_project()
            .await
            .expect("Error getting project");
        project_mock.assert();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn check_version_parsing() {
        let version_mock = mock("GET", "/v2/projects/paper/versions/1.16.5")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
              "project_id": "paper",
              "project_name": "Paper",
              "version": "1.16.5",
              "builds": [
                463,
                464,
                465,
                466
              ]
            }"#,
            )
            .create();
        let expected = Version {
            builds: vec![463, 464, 465, 466],
        };
        let actual = get_paper_api()
            .get_version("1.16.5")
            .await
            .expect("Error getting version");
        version_mock.assert();
        assert_eq!(actual, expected);
    }

    #[tokio::test]
    async fn check_build_parsing() {
        let build_mock = mock("GET", "/v2/projects/paper/versions/1.16.5/builds/466")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
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
            }"#,
            )
            .create();
        let expected = VersionBuild {
            build: 466,
            time: Utc.ymd(2021, 02, 08).and_hms_milli(10, 22, 13, 662),
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
        let actual = get_paper_api()
            .get_build("1.16.5", 466)
            .await
            .expect("Error getting build");
        build_mock.assert();
        assert_eq!(actual, expected);
    }

    // TODO: Parameterize further
    fn get_version_mock(build: BuildNumber) -> mockito::Mock {
        mock("GET", "/v2/projects/paper/versions/1.16.5")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(format!(
                r#"{{
              "project_id": "paper",
              "project_name": "Paper",
              "version": "1.16.5",
              "builds": [ {} ]
            }}"#,
                build
            ))
    }

    #[tokio::test]
    async fn check_stream_update() {
        let mut stream = PaperStream::new(get_paper_api(), "1.16.5");
        {
            let version_mock = get_version_mock(5).create();
            let update = stream.fetch_update().await.expect("Update fetch should succeed");
            version_mock.assert();
            assert_eq!(update, None);
        }
        {
            let version_mock = get_version_mock(6).create();
            let update = stream.fetch_update().await.expect("Update fetch should succeed");
            version_mock.assert();
            assert!(matches!(update, Some(_)));
        }
    }
}
