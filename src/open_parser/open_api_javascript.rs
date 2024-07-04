use super::parser_tools::{OpenApiModule, OpenApiRequester};
use crate::{
    command_config::CommandConfig,
    open_api::open_api_3::{
        Open3ApiConfig, Open3Components, Open3ComponentsSchema, Open3Config, Open3Requests,
        Open3Schema,
    },
    tools::tools::capitalize,
};
use lazy_static::lazy_static;
use regex::Regex;
use std::collections::HashMap;

pub trait OpenApiJavaScriptParser {
    /// 获取模块列表
    /// 可利用模块列表生成模块文件或文件夹对接口进行分类
    fn get_module_list(&self) -> Vec<OpenApiModule>;

    /// 返回api列表
    fn get_api_list(&self) -> &Vec<(String, OpenApiRequester)>;

    /// 获取所有类型列表
    fn get_interface_enum_list(&self, ignore_option: &bool) -> Vec<String>;
}

pub struct OpenApi3JavaScript<'a, 'b> {
    config: &'a mut Open3Config,
    command_config: &'b CommandConfig,
    api_list: Vec<(String, OpenApiRequester)>,
}

impl<'a, 'b> OpenApi3JavaScript<'a, 'b> {
    pub fn new(
        config: &'a mut Open3Config,
        command_config: &'b CommandConfig,
    ) -> OpenApi3JavaScript<'a, 'b> {
        let api_list = open_3_get_api_list(config, command_config);
        OpenApi3JavaScript {
            config,
            api_list,
            command_config,
        }
    }
}

impl OpenApiJavaScriptParser for OpenApi3JavaScript<'_, '_> {
    fn get_module_list(&self) -> Vec<OpenApiModule> {
        self.config
            .tags
            .iter()
            .filter_map(|v| {
                let name = v.name.to_string();
                if !self.command_config.tags.is_empty() && !self.command_config.tags.contains(&name)
                {
                    None
                } else {
                    Some(OpenApiModule {
                        description: v.description.to_string(),
                        name: v.name.to_string(),
                    })
                }
            })
            .collect()
    }

    fn get_api_list(&self) -> &Vec<(String, OpenApiRequester)> {
        &self.api_list
    }

    /// 获取所有的接口列表
    fn get_interface_enum_list(&self, ignore_option: &bool) -> Vec<String> {
        let mut str_vec = vec![];

        let mut components_schema_vec: Vec<(&String, &Open3ComponentsSchema)> =
            self.config.components.schemas.iter().collect();
        components_schema_vec.sort_by(|a, b| a.0.cmp(&b.0));

        let request_scheme_name_vec: Vec<String> = self
            .api_list
            .iter()
            .map(|v| v.1.request_schema_name.to_string())
            .collect();

        for (_, schema) in components_schema_vec {
            str_vec.push(open_3_create_ts_interface_enum(
                schema,
                &request_scheme_name_vec,
                &self.command_config.namespace,
                ignore_option,
            ));
        }
        str_vec
    }
}

/// 将open api类型转成ts对应的类型
pub fn ts_type_transform(data_type: &str) -> String {
    lazy_static! {
        static ref JS_TYPE_MAP: HashMap<&'static str, &'static str> = {
            let mut m = HashMap::new();
            m.insert("array", "Array");
            m.insert("number", "number");
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

/// 处理OpenApi3的类型
///
/// 转换成易处理的 OpenApiRequester 类型
///
/// 目前仅支持get post delete put
fn open_3_get_api_list(
    config: &mut Open3Config,
    command_config: &CommandConfig,
) -> Vec<(String, OpenApiRequester)> {
    let mut paths_vec: Vec<(&String, &Open3Requests)> = config.paths.iter().collect();
    paths_vec.sort_by(|a, b| a.0.cmp(b.0));
    let mut api_list: Vec<(String, OpenApiRequester)> = vec![];
    for (url, requests) in paths_vec {
        for (method, request) in requests.iter() {
            if let Some(api_config) = request {
                let module = &api_config.tags[0];
                let operation_id = &api_config.operation_id;
                let request_type = open_3_get_request_type_name(
                    &method,
                    &mut config.components,
                    &api_config,
                    command_config,
                );
                // 如果已经指定了tag， 则忽略其他tag
                if !command_config.tags.is_empty() {
                    if !command_config.tags.contains(&module) {
                        continue;
                    }
                }
                api_list.push((
                    module.to_string(),
                    OpenApiRequester {
                        summary: if let Some(summary) = &api_config.summary {
                            summary.to_string()
                        } else {
                            String::new()
                        },
                        method: method.clone(),
                        operation_id: format!(
                            "{}{}",
                            command_config
                                .operation_prefix
                                .as_ref()
                                .unwrap_or(&"".to_string()),
                            operation_id.to_string()
                        ),
                        url: url.to_string(),
                        request_schema_name: request_type.0,
                        // 可作为调用方法的参数类型， 已经处理是否可选
                        request_type_name: request_type.1,
                        // content_type: String::from("application/json"),
                        response_type_name: open_3_get_response_type_name(
                            &api_config,
                            command_config,
                        ),
                        is_form: api_config.parameters.is_some() && method.eq("post"),
                    },
                ))
            }
        }
    }
    api_list
}

/// 获取响应类型名称
fn open_3_get_response_type_name(
    api_config: &Open3ApiConfig,
    command_config: &CommandConfig,
) -> String {
    if let Some(response_content_map) = api_config
        .responses
        .get("200")
        .and_then(|x| x.content.as_ref())
    {
        for (_, response_content) in response_content_map {
            if let Some(response_content) = response_content {
                let schema = &response_content.schema;
                return open_3_get_type_name_from_schema(
                    schema,
                    command_config.namespace.clone(),
                    "",
                );
            }
        }
    }
    String::from("void")
}

/// 获取请求参数类型名称
///
/// 如若当前的请求参数类型是可选的 则会在类型后面拼接 | void，
/// 将拼接结果作为第二个参数返回
///
/// 如果命令行参数指定了namespace 则会将namespace拼接在类型前
///
/// 对于get/delete请求
///
/// - 根据api_config判断当前请求类型
/// - 如果使用path或query参数
/// - 将path和query参数混合生成新的类型塞进components（可使用operation_id + 关键字作为地址参数的类型名称）
fn open_3_get_request_type_name(
    method: &str,
    components: &mut Open3Components,
    api_config: &Open3ApiConfig,
    command_config: &CommandConfig,
) -> (String, String) {
    if vec!["get", "delete"].contains(&method) {
        generate_get_request_type(components, api_config, command_config)
    } else {
        generate_post_request_type(components, api_config, command_config)
    }
}

/// 生成get请求的请求类型并塞进components
fn generate_get_request_type(
    components: &mut Open3Components,
    api_config: &Open3ApiConfig,
    command_config: &CommandConfig,
) -> (String, String) {
    let schemas = &mut components.schemas;
    let mut properties = HashMap::new();
    let type_name = format!("{}Query", capitalize(&api_config.operation_id));
    let type_name_with_namespace = if let Some(namespace) = &command_config.namespace {
        format!("{}.{}", namespace, type_name)
    } else {
        type_name.clone()
    };
    let mut required_vec = Vec::new();
    if let Some(parameters) = &api_config.parameters {
        parameters.iter().for_each(|v| {
            properties.insert(
                v.name.clone(),
                Open3Schema {
                    schema_type: Some("string".to_string()),
                    schema_ref: None,
                    items: None,
                    property_enum: None,
                    format: None,
                    description: v.description.clone(),
                },
            );
            if v.required {
                required_vec.push(v.name.clone());
            }
        });
    }
    let required_vec_is_empty = required_vec.is_empty();
    let properties_is_empty = properties.is_empty();
    if properties_is_empty {
        return (String::from("void"), String::from("void"));
    }
    let components_schema = Open3ComponentsSchema {
        title: type_name.clone(),
        schema_type: "object".to_string(),
        properties: Some(properties),
        required: Some(required_vec),
    };
    schemas.insert(type_name.clone(), components_schema);
    (
        type_name_with_namespace.clone(),
        if !required_vec_is_empty {
            type_name_with_namespace
        } else {
            format!("{} | void", type_name_with_namespace)
        },
    )
}

/// 获取post请求的请求类型
fn generate_post_request_type(
    components: &Open3Components,
    api_config: &Open3ApiConfig,
    command_config: &CommandConfig,
) -> (String, String) {
    if let Some(schema_ref) = api_config
        .request_body
        .as_ref()
        .and_then(|x| x.content.get("application/json"))
        .and_then(|x| x.schema.schema_ref.as_ref())
    {
        let is_required = open_3_schema_is_required(components, schema_ref);
        let type_name = open_3_get_type_name_from_schema_ref(schema_ref);
        // 如果指定namespace 则需要在类型前添加namespace.
        let type_name = if let Some(namespace) = &command_config.namespace {
            format!("{}.{}", namespace, type_name)
        } else {
            type_name
        };
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
///
/// 如果命令行参数指定了namesapce 则会将namespace拼接在类型前
fn open_3_get_type_name_from_schema(
    schema: &Open3Schema,
    namespace: Option<String>,
    generic: &str,
) -> String {
    lazy_static! {
        static ref SCHEMA_GENERIC_REGEX: Regex = Regex::new(r"<T>").unwrap();
    }
    if let Some(schema_ref) = &schema.schema_ref {
        let schema_ref = open_3_get_type_name_from_schema_ref(schema_ref);
        let schema_ref = if let Some(namespace) = namespace {
            format!("{}.{}", namespace, schema_ref)
        } else {
            schema_ref
        };
        if generic.is_empty() {
            return schema_ref;
        }
        return SCHEMA_GENERIC_REGEX
            .replace_all(generic, format!("<{}>", schema_ref))
            .to_string();
    }
    if let Some(schema_type) = &schema.schema_type {
        if schema_type.eq("array") {
            return open_3_get_type_name_from_schema(
                schema.items.as_ref().unwrap(),
                namespace,
                "Array<T>",
            );
        }
        let translate_type = ts_type_transform(&schema_type.clone());
        if generic.is_empty() {
            return translate_type;
        }
        return SCHEMA_GENERIC_REGEX
            .replace_all(generic, format!("<{}>", translate_type))
            .to_string();
    }
    String::from("void")
}

/// 根据schema ref获取schema名称
///
/// 如：#/components/schemas/Result«User»
/// 则返回 Result«User»
fn get_schema_name_from_schema_ref(schema_ref: &str) -> String {
    lazy_static! {
        static ref SCHEMA_NAME_REGEX: Regex = Regex::new(r".*/").unwrap();
    }
    SCHEMA_NAME_REGEX.replace_all(schema_ref, "").to_string()
}

/// 根据schema ref获取类型名称
///
/// 如：#/components/schemas/Result«User»
/// 则返回 ResultUser
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
fn open_3_schema_is_required(components: &Open3Components, scheme_ref: &str) -> bool {
    let schema_name = get_schema_name_from_schema_ref(scheme_ref);
    if let Some(required) = components
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
    request_type_name_vec: &Vec<String>,
    namespace: &Option<String>,
    ignore_option: &bool,
) -> String {
    let interface_name = open_3_get_type_name_from_schema_ref(&components_schema.title);
    let mut interface_str = format!("interface {} {{", &interface_name);
    let mut open_api_schema_vec: Vec<(&String, &Open3Schema)> =
        if let Some(properties) = &components_schema.properties {
            properties.into_iter().collect()
        } else {
            vec![]
        };
    open_api_schema_vec.sort_by(|a, b| a.0.cmp(&b.0));
    let required_default_vec = vec![];
    let required_vec: &Vec<String> = if let Some(required_vec) = &components_schema.required {
        required_vec
    } else {
        &required_default_vec
    };
    let interface_with_namespace = if let Some(namespace) = namespace {
        format!("{}.{}", namespace, interface_name)
    } else {
        interface_name.to_string()
    };
    let is_request_name_interface = request_type_name_vec.contains(&interface_with_namespace);
    let ignore_option = !is_request_name_interface && *ignore_option;
    for (property_name, property) in open_api_schema_vec.iter() {
        let property_option_split = if ignore_option || required_vec.contains(property_name) {
            ""
        } else {
            "?"
        };
        let schema_type = open_3_get_type_name_from_schema(property, None, "");
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
    // 对于空的interface，添加string unknown的签名
    if open_api_schema_vec.is_empty() {
        interface_str.push_str(
            r"
  [key:string]:unknown;",
        );
    }
    interface_str.push_str("\n}\n\n");
    interface_str
}
