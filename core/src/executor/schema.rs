use serde::Deserialize;
use std::fs::File;
use std::io::{BufReader, Read};
use std::{collections::HashMap, path::Path};

// Use anyhow for simple error handling in the example
use anyhow::{Context, Result};

/// Represents the entire API test file structure.
#[derive(Debug, Deserialize, Default)]
pub struct Schema {
    #[serde(default)]
    pub filename: String,
    #[serde(default)] // Make imports optional
    pub imports: Vec<String>,
    #[serde(default)] // Make env optional
    pub env: HashMap<String, EnvironmentVariable>,
    #[serde(default)] // Make requests optional
    pub requests: HashMap<String, Request>,
    #[serde(default)] // Make calls optional
    pub calls: HashMap<String, Vec<String>>,
    /// Project defination, just for more information
    #[serde(default)]
    pub project: Option<Project>,
}

/// Used to describe the project from a root file.
/// Might contain project configurations too
#[derive(Debug, Deserialize, Clone, Default)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub authors: Vec<User>,
    /// This is the openapi file that this project will be continousely generated from if specified
    #[serde(default)]
    pub generator: Option<String>,
    /// The environment to be run on by default
    #[serde(default)]
    pub default_env: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct User {
    pub name: String,
    #[serde(default)]
    pub email: String,
}

/// Represents the definition of a single environment variable.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct EnvironmentVariable {
    pub default: serde_yaml::Value, // Use Value to allow any YAML type
    #[serde(flatten)] // Flatten environment-specific overrides into this struct
    pub overrides: HashMap<String, serde_yaml::Value>,
}

/// Represents a single API request definition.
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Request {
    pub method: String,
    pub url: String,
    #[serde(default)]
    pub doc: String,
    #[serde(default)]
    pub config: Option<RequestConfig>, // Optional config block
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>, // Optional headers block
    #[serde(default)]
    pub query: Option<HashMap<String, String>>, // Optional query block, values can be complex
    #[serde(default)]
    pub body: Option<RequestBody>, // Optional body block
    #[serde(default)]
    pub script: Option<RequestScriptConfig>, // Optional script block
}

/// Represents the configuration section of a request.
#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "snake_case")]
pub struct RequestConfig {
    #[serde(default)]
    pub depends_on: Vec<String>, // defaults to empty vec if not present
    pub delay: Option<String>,   // e.g., "500ms", "1s"
    pub timeout: Option<String>, // e.g., "30s"
    #[serde(default)] // default to 0 if not present
    pub retries: u32,
}

/// Represents the script section of a request.
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct RequestScriptConfig {
    pub post_request: Option<Script>,
    pub pre_request: Option<Script>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case", tag = "language")]
pub enum Script {
    #[serde(rename = "lua")]
    Lua { content: String },
    #[serde(rename = "javascript")]
    Javascript { content: String },
    #[serde(rename = "rhai")]
    Rhai { content: String },
}

/// Represents the body section of a request.
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")] // Use 'type' field to determine which variant to deserialize
pub enum RequestBody {
    #[serde(rename = "json")]
    Json {
        content: serde_yaml::Value, // Use Value to allow any JSON structure (object or array)
    },
    #[serde(rename = "graphql")]
    Graphql {
        query: String,
        variables: Option<serde_yaml::Value>, // GraphQL variables as a JSON-like structure
    },
    #[serde(rename = "xml")]
    Xml {
        content: String, // XML content as a string
    },
    #[serde(rename = "text")]
    Text {
        content: String, // Text content as a string
    },
    #[serde(rename = "form-urlencoded")]
    FormUrlencoded {
        content: String, // Form URL-encoded string
    },
    #[serde(rename = "multipart")]
    Multipart {
        parts: Vec<MultipartPart>, // List of multipart parts
    },
}

/// Represents a single part within a multipart request body.
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")] // Use 'kind' field to determine field or file
pub enum MultipartPart {
    #[serde(rename = "field")]
    Field { name: String, value: String },
    #[serde(rename = "file")]
    File {
        name: String,
        path: String,
        mime_type: Option<String>, // Optional MIME type
    },
}

/// Parses a YAML string into an Schema struct.
pub fn parse_api_yaml(yaml_content: &str) -> Result<Schema> {
    return serde_yaml::from_str(yaml_content).context("Failed to parse API test YAML");
}

pub fn parse_api_test_yaml_reader<R: Read>(reader: R) -> Result<Schema> {
    return serde_yaml::from_reader(reader).context("Failed to parse API test YAML from reader");
}

pub fn load_api_file(path: &Path) -> Result<Schema> {
    let file = File::open(path).context("Failed to open API test file")?;
    let reader = BufReader::new(file);
    let mut schema =
        parse_api_test_yaml_reader(reader).context("Failed to parse content from API test file")?;

    // include file information
    schema.filename = path
        .to_str()
        .context("Could not resolve rootpath as string")?
        .to_string();

    return Ok(schema);
}
