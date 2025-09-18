use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;
use ts_rs::TS;

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct CustomPrompt {
    pub name: String,                  // "command" or "directory:command"
    pub path: PathBuf,                 // full path to the file
    pub content: String,               // file contents (frontmatter stripped)
    pub category: Option<String>,      // directory name for organization
    pub argument_hint: Option<String>, // argument-hint from frontmatter
    // New fields for template support
    pub template_args: Option<Vec<TemplateArg>>,
    pub template_syntax: Option<TemplateSyntax>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub struct TemplateArg {
    pub name: String,
    pub description: Option<String>,
    pub required: bool,
    pub default_value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, TS)]
pub enum TemplateSyntax {
    Simple,
    Askama,
}
