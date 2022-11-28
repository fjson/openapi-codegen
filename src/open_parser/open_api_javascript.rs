use super::parser_tools::{OpenApiModule, OpenApiResquest};
use crate::open_api::open_api_3::{
    Open3ApiConfig, Open3ComponentsSchema, Open3Config, Open3Requests, Open3Schema,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

pub trait OpenApiJavaScriptParser {
    /// 获取模块列表
    /// 可利用模块列表生成模块文件或文件夹对接口进行分类
    fn get_module_list(&self) -> Vec<OpenApiModule>;

    /// 返回api列表
    fn get_api_list(&self) -> &Vec<(String, OpenApiResquest)>;

    /// 获取类型列表
    fn get_interface_enum_list(&self, ignore_option: &bool) -> Vec<String>;
}

pub struct OpenApi3JavaScript<'a> {
    config: &'a Open3Config,
    api_list: Vec<(String, OpenApiResquest)>,
}

impl<'a> OpenApi3JavaScript<'a> {
    pub fn new(config: &'a Open3Config) -> OpenApi3JavaScript<'a> {
        OpenApi3JavaScript {
            config,
            api_list: open_3_get_api_list(config),
        }
    }
}

impl OpenApiJavaScriptParser for OpenApi3JavaScript<'_> {
    fn get_module_list(&self) -> Vec<OpenApiModule> {
        self.config
            .tags
            .iter()
            .map(|v| OpenApiModule {
                description: v.description.to_string(),
                name: v.name.to_string(),
            })
            .collect()
    }

    fn get_api_list(&self) -> &Vec<(String, OpenApiResquest)> {
        &self.api_list
    }

    fn get_interface_enum_list(&self, ignore_option: &bool) -> Vec<String> {
        let mut str_vec = vec![];

        let mut components_schema_vec: Vec<(&String, &Open3ComponentsSchema)> =
            self.config.components.schemas.iter().collect();
        components_schema_vec.sort_by(|a, b| a.0.cmp(&b.0));

        let request_scheme_name_vec: Vec<&String> = self
            .api_list
            .iter()
            .map(|v| &v.1.request_schema_name)
            .collect();

        for (_, schema) in components_schema_vec {
            str_vec.push(open_3_create_ts_interface_enum(
                schema,
                &request_scheme_name_vec,
                ignore_option,
            ));
        }
        str_vec
    }
}

/// 支持将open api类型转成ts对应的类型
pub fn ts_data_type_translate(data_type: &str) -> String {
    lazy_static! {
        static ref JS_TYPE_MAP: HashMap<&'static str, &'static str> = {
            let mut m = HashMap::new();
            m.insert("array", "Array");
            m.insert("int", "number");
            m.insert("integer", "number");
            m.insert("double", "number");
            m.insert("float", "number");
            m.insert("long", "number");
            m.insert("short", "number");
            m.insert("char", "string");
            m.insert("object", "any");
            m.insert("Map", "any");
            m.insert("date", "string");
            m.insert("DateTime", "string");
            m.insert("binary", "string");
            m.insert("File", "any");
            m.insert("string", "string");
            m.insert("boolean", "boolean");
            m
        };
    }
    if let Some(data_type) = JS_TYPE_MAP.get(data_type) {
        data_type.to_string()
    } else {
        "void".to_string()
    }
}

fn open_3_get_api_list(config: &Open3Config) -> Vec<(String, OpenApiResquest)> {
    let mut paths_vec: Vec<(&String, &Open3Requests)> = config.paths.iter().collect();
    paths_vec.sort_by(|a, b| a.0.cmp(b.0));
    let mut api_list: Vec<(String, OpenApiResquest)> = vec![];
    for (url, requests) in paths_vec {
        for (method, request) in requests.iter() {
            if let Some(api_config) = request {
                let module = &api_config.tags[0];
                let operation_id = &api_config.operation_id;
                let request_type = open_3_get_request_type_name(&config, &api_config);
                api_list.push((
                    module.to_string(),
                    OpenApiResquest {
                        summary: if let Some(summary) = &api_config.summary {
                            summary.to_string()
                        } else {
                            String::new()
                        },
                        method,
                        operation_id: operation_id.to_string(),
                        url: url.to_string(),
                        request_schema_name: request_type.0,
                        request_type_name: request_type.1,
                        content_type: String::from("application/json"),
                        response_type_name: open_3_get_response_type_name(&api_config),
                    },
                ))
            }
        }
    }
    api_list
}

/// 获取响应类型名称
fn open_3_get_response_type_name(api_config: &Open3ApiConfig) -> String {
    if let Some(response_content_map) = api_config
        .responses
        .get("200")
        .and_then(|x| x.content.as_ref())
    {
        for (_, response_content) in response_content_map {
            if let Some(response_content) = response_content {
                let schema = &response_content.schema;
                return open_3_get_type_name_from_schema(schema, "");
            }
        }
    }
    String::from("void")
}

/// 获取请求参数类型名称
fn open_3_get_request_type_name(
    open_config: &Open3Config,
    api_config: &Open3ApiConfig,
) -> (String, String) {
    if let Some(schema_ref) = api_config
        .request_body
        .as_ref()
        .and_then(|x| x.content.get("application/json"))
        .and_then(|x| x.schema.schema_ref.as_ref())
    {
        let is_required = open_3_schema_is_required(open_config, schema_ref);
        let type_name = open_3_get_type_name_from_schema_ref(schema_ref);
        let mut type_name_clone = type_name.clone();
        if !type_name.eq("void") {
            return (
                type_name,
                if !is_required {
                    type_name_clone.push_str(" | void");
                    type_name_clone
                } else {
                    type_name_clone
                },
            );
        }
    }
    (String::from("void"), String::from("void"))
}

/// 根据schema生成响应类型名称
fn open_3_get_type_name_from_schema(schema: &Open3Schema, generic: &str) -> String {
    if let Some(schema_ref) = &schema.schema_ref {
        let schema_ref = open_3_get_type_name_from_schema_ref(schema_ref);
        if generic.is_empty() {
            return schema_ref;
        }
        lazy_static! {
            static ref SCHEMA_GENERIC_REGEX: Regex = Regex::new(r"<T>").unwrap();
        }
        return SCHEMA_GENERIC_REGEX
            .replace_all(generic, format!("<{}>", schema_ref))
            .to_string();
    }
    if let Some(schema_type) = &schema.schema_type {
        if schema_type.eq("array") {
            return open_3_get_type_name_from_schema(schema.items.as_ref().unwrap(), "Array<T>");
        }
        return ts_data_type_translate(&schema_type.clone());
    }
    String::from("void")
}

/// 根据schema ref获取schema名称
fn get_schema_name_from_schema_ref(schema_ref: &str) -> String {
    lazy_static! {
        static ref SCHEMA_NAME_REGEX: Regex = Regex::new(r".*/").unwrap();
    }
    SCHEMA_NAME_REGEX.replace_all(schema_ref, "").to_string()
}

/// 根据schema ref获取类型名称
fn open_3_get_type_name_from_schema_ref(schema_ref: &str) -> String {
    lazy_static! {
        static ref SCHEMA_TYPE_NAME_REGEX: Regex = Regex::new(r"[«»]").unwrap();
    }
    let schema_name = get_schema_name_from_schema_ref(schema_ref);
    SCHEMA_TYPE_NAME_REGEX
        .replace_all(&schema_name, "")
        .to_string()
}

/// 判断 schema 参数是否是必须的
fn open_3_schema_is_required(open_config: &Open3Config, scheme_ref: &str) -> bool {
    let schema_name = get_schema_name_from_schema_ref(scheme_ref);
    if let Some(required) = open_config
        .components
        .schemas
        .get(&schema_name)
        .and_then(|x| x.required.as_ref())
    {
        return !required.is_empty();
    }
    false
}

/// 生成typescript interface类型
fn open_3_create_ts_interface_enum(
    components_schema: &Open3ComponentsSchema,
    request_type_name_vec: &Vec<&String>,
    ignore_option: &bool,
) -> String {
    let interface_name = open_3_get_type_name_from_schema_ref(&components_schema.title);
    let mut interface_str = format!("interface {} {{", &interface_name);

    let mut open_api_schema_vec: Vec<(&String, &Open3Schema)> =
        components_schema.properties.iter().collect();
    open_api_schema_vec.sort_by(|a, b| a.0.cmp(&b.0));

    let required_default_vec = vec![];
    let required_vec = if let Some(required_vec) = &components_schema.required {
        required_vec
    } else {
        &required_default_vec
    };

    let is_request_name_interface = request_type_name_vec.contains(&&interface_name);
    let ignore_option = !is_request_name_interface && *ignore_option;

    for (property_name, property) in open_api_schema_vec {
        let property_option_split = if ignore_option || required_vec.contains(property_name) {
            ""
        } else {
            "?"
        };
        let schema_type = open_3_get_type_name_from_schema(property, "");
        let description = if let Some(description) = &property.description {
            description
        } else {
            ""
        };
        let interface_item = format!(
            r"
  /**
   * {description}
   * @type {schema_type}
   * @memberof {interface_name}
   */
  {property_name}{property_option_split}: {schema_type};"
        );
        interface_str.push_str(&interface_item);
    }
    interface_str.push_str("\n}\n\n");
    interface_str
}
