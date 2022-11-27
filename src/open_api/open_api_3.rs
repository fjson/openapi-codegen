use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Open3Config {
    pub components: Open3Components,
    pub paths: HashMap<String, Open3Requests>,
    pub tags: Vec<Open3Tag>,
}

type Open3ApiConfigOption = Option<Open3ApiConfig>;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Open3Requests {
    pub get: Open3ApiConfigOption,
    pub post: Open3ApiConfigOption,
    pub put: Open3ApiConfigOption,
    pub delete: Open3ApiConfigOption,
}

impl Open3Requests {
    pub fn iter(&self) -> Open3RequestsIntoIterator {
        Open3RequestsIntoIterator {
            config: self,
            index: 0,
        }
    }
}

pub struct Open3RequestsIntoIterator<'a> {
    config: &'a Open3Requests,
    index: usize,
}

impl<'a> Iterator for Open3RequestsIntoIterator<'a> {
    type Item = (String, &'a Option<Open3ApiConfig>);
    fn next(&mut self) -> Option<(String, &'a Open3ApiConfigOption)> {
        let result = match self.index {
            0 => (String::from("get"), &self.config.get),
            1 => (String::from("post"), &self.config.post),
            2 => (String::from("put"), &self.config.put),
            3 => (String::from("delete"), &self.config.delete),
            _ => return None,
        };
        self.index += 1;
        Some(result)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Open3ApiConfig {
    // 操作名称
    #[serde(alias = "operationId")]
    pub operation_id: String,

    // 响应配置
    pub responses: HashMap<String, Open3Response>,

    // 请求配置
    #[serde(alias = "requestBody")]
    pub request_body: Option<Open3RequestBody>,

    pub summary: String,

    pub tags: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Open3RequestBody {
    pub content: HashMap<String, Open3ResponseContent>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Open3RequestBodyContent {
    pub schema: Open3RequestBodyContentSchema,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Open3RequestBodyContentSchema {
    #[serde(alias = "$ref")]
    pub schema_ref: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Open3Response {
    pub content: Option<HashMap<String, Option<Open3ResponseContent>>>,
    // 响应描述
    pub description: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Open3ResponseContent {
    pub schema: Open3Schema,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Open3Schema {
    #[serde(alias = "$ref")]
    pub schema_ref: Option<String>,

    #[serde(alias = "type")]
    pub schema_type: Option<String>,

    pub items: Option<Box<Open3Schema>>,

    #[serde(alias = "enum")]
    pub property_enum: Option<Vec<String>>,

    pub format: Option<String>,

    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Open3Tag {
    // tag 描述
    pub description: String,
    // tag 名称
    pub name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Open3Components {
    pub schemas: HashMap<String, Open3ComponentsSchema>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Open3ComponentsSchema {
    pub title: String,

    #[serde(alias = "type")]
    pub schema_type: String,

    pub properties: HashMap<String, Open3Schema>,

    pub required: Option<Vec<String>>,
}
