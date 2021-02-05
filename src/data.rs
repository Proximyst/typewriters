use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct HeadData {
    commit: String,
    build: String,
    version: String,
}
