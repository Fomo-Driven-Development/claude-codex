use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct CustomPrompt {
    pub name: String,             // "command" or "directory:command"
    pub path: PathBuf,            // full path to the file
    pub content: String,          // file contents
    pub category: Option<String>, // directory name for organization
}
