use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};
use log::info;
use crate::{
    command_config::CommandConfig,
    open_parser::{
        open_api_javascript::OpenApiJavaScriptParser,
        parser_tools::{OpenApiModule, OpenApiRequester},
    },
};

/// 生成 open api typescript调用
pub fn create_typescript_api(
    command_config: &CommandConfig,
    open_api_parser: &impl OpenApiJavaScriptParser,
) {
    init_workspace(command_config);
    create_default_resource_file(command_config);
    create_ts_d_ts(command_config, open_api_parser);
    create_entry_file(command_config, open_api_parser);
    create_controller(command_config, open_api_parser);
}

/// 初始化工作空间
pub fn init_workspace(command_config: &CommandConfig) {
    info!("init workspace");
    fs::create_dir_all(&command_config.workspace).expect("init workspace error");
}

/// 创建api入口文件
pub fn create_entry_file(
    command_config: &CommandConfig,
    open_api_parser: &impl OpenApiJavaScriptParser,
) {
    info!("create entry file");
    let workspace_path = Path::new(&command_config.workspace);
    let entry_file_path = workspace_path.join("index.ts");

    // 创建并打开入口文件
    let mut f = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(entry_file_path)
        .expect("cannot open entry file");

    // 写入入口文件内容
    for module in open_api_parser.get_module_list(command_config) {
        let write_content =
            create_entry_export_template(&command_config.controller_dir_name, &module);
        f.write_all(write_content.as_bytes())
            .expect(format!("{} write error", write_content).as_str());
    }
}

/// 创建接口调用
fn create_controller(
    command_config: &CommandConfig,
    open_api_parser: &impl OpenApiJavaScriptParser,
) {
    let workspace_path = Path::new(&command_config.workspace);
    let controller_dir_path = workspace_path.join(&command_config.controller_dir_name);
    fs::create_dir_all(&controller_dir_path).expect("create controller dir error");

    // 存储所有的路径
    let mut module_path_map = HashMap::new();

    for module in open_api_parser.get_module_list(command_config) {
        let module_dir_path = controller_dir_path.as_path().join(&module.name);
        if let Some(module_dir_path) = module_dir_path.to_str() {
            module_path_map.insert(module.name, String::from(module_dir_path));
        }
    }

    // 文件拆分模式，优先创建 module文件夹
    if command_config.split {
        // 创建module文件夹
        for (_, dir) in &module_path_map {
            fs::create_dir_all(&dir).expect(format!("create module {} dir error", &dir).as_str());
        }
    }

    let mut file_touched_record_map = HashMap::new();

    for (module, request) in open_api_parser.get_api_list() {
        if let Some(module_path) = module_path_map.get(module) {
            let operation_id = &request.operation_id;
            info!("generate call {}  ", operation_id);
            let api_template = create_api_call(&request);
            let file_end = ".ts";

            // 生成写入文件路径
            let file_path = if command_config.split {
                let module_path_buf = Path::new(&module_path);
                module_path_buf.join(format!("{}{}", operation_id, file_end))
            } else {
                let module_path_buf = PathBuf::new();
                module_path_buf.join(format!("{}{}", module_path, file_end))
            };

            // 文件拆分模式下 需要生成模块的入口文件
            if command_config.split {
                let module_entry_path = Path::new(&module_path).join("index.ts");
                let module_entry_path_str_key =
                    if let Some(module_entry_path_str_key) = module_entry_path.clone().to_str() {
                        String::from(module_entry_path_str_key)
                    } else {
                        String::new()
                    };

                // 判断文件是否已经清空打开过
                let file_touched =
                    if let Some(flag) = file_touched_record_map.get(&module_entry_path_str_key) {
                        *flag
                    } else {
                        false
                    };

                // 打开模块的入口文件
                let mut entry_f = if module_entry_path.exists() && file_touched {
                    OpenOptions::new()
                        .append(true)
                        .open(module_entry_path)
                        .expect(format!("{} open error", module_entry_path_str_key).as_str())
                } else {
                    OpenOptions::new()
                        .create(true)
                        .truncate(true)
                        .write(true)
                        .open(module_entry_path)
                        .expect(format!("{} open error", module_entry_path_str_key).as_str())
                };

                // 记录module入口文件是否被打开过
                if !module_entry_path_str_key.is_empty() {
                    file_touched_record_map.insert(module_entry_path_str_key.clone(), true);
                }

                // 写入内容
                entry_f
                    .write_all(
                        {
                            let mut content = format!(r#"export * from "./{operation_id}";"#);
                            content.push_str("\n");
                            content
                        }
                        .as_bytes(),
                    )
                    .expect(format!("{} write error", module_entry_path_str_key).as_str());
            }

            let file_path_str_key = if let Some(file_path_str_key) = file_path.clone().to_str() {
                String::from(file_path_str_key)
            } else {
                String::new()
            };

            // 判断文件是否已经清空打开过
            let file_touched = if let Some(flag) = file_touched_record_map.get(&file_path_str_key) {
                *flag
            } else {
                false
            };

            // 打开目标文件
            let mut f = if file_path.exists() && file_touched {
                OpenOptions::new()
                    .append(true)
                    .open(file_path)
                    .expect(format!("{} open error", operation_id).as_str())
            } else {
                OpenOptions::new()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(file_path)
                    .expect(format!("{} open error", operation_id).as_str())
            };

            let api_template = if command_config.split || !file_touched {
                let mut api_import_temp = create_api_import(command_config);
                api_import_temp.push_str(&api_template);
                api_import_temp
            } else {
                api_template
            };

            // 记录文件被打开过
            if !file_path_str_key.is_empty() {
                file_touched_record_map.insert(file_path_str_key, true);
            }

            // 写入内容
            f.write_all(api_template.as_bytes())
                .expect(format!("{} write error", operation_id).as_str());
        }
    }
}

/// 生成接口导出项
fn create_entry_export_template(controller_dir_name: &str, tag: &OpenApiModule) -> String {
    let module = &tag.name;
    let desc = &tag.description;
    String::from(format!(
        r#"// {desc} 
export * from './{controller_dir_name}/{module}';
"#
    ))
}

/// 生成api调用
fn create_api_call(open_api_request: &OpenApiRequester) -> String {
    // 接口调用名
    let operation_id = &open_api_request.operation_id;
    // 接口说明
    let summary = &open_api_request.summary;
    // 响应类型
    let response_type = &open_api_request.response_type_name;
    let request_type = &open_api_request.request_type_name;
    let method = &open_api_request.method;
    let api_url = &open_api_request.url;
    let method_name = if summary.starts_with("[No Auth]") {
        format!("{method}NoAuth")
    }else {
        method.to_string()
    };
    format!(
        r#"
/**
 * {summary}
 */
export function {operation_id}(req:{request_type}, config?: RequestConfig): Promise<{response_type}> {{
    return resource.{method_name}("{api_url}", req, config);
}}
    "#
    )
}

/// 创建api调用文件中的导入内容
fn create_api_import(command_config: &CommandConfig) -> String {
    let import_path = if command_config.split {
        "../../"
    } else {
        "../"
    };
    format!(
        r#"import {{ resource, RequestConfig }} from "{import_path}helper/resource";
"#
    )
}

/// 生成typescript类型文件
fn create_ts_d_ts(command_config: &CommandConfig, open_api_parser: &impl OpenApiJavaScriptParser) {
    info!("create api.d.ts");
    let workspace_path = Path::new(&command_config.workspace);
    let mut ts_d_f = OpenOptions::new()
        .create(true)
        .truncate(true)
        .write(true)
        .open(workspace_path.join("api.d.ts"))
        .expect("ts .d open error");
    for value in open_api_parser.get_interface_enum_list(&command_config.ignore_option) {
        ts_d_f
            .write_all(value.as_bytes())
            .expect(format!("{} td type write error", value).as_str());
    }
}

/// 创建默认的调用文件
fn create_default_resource_file(command_config: &CommandConfig) {
    let workspace_path = Path::new(&command_config.workspace);
    let helper_dir_path = workspace_path.join("helper");
    fs::create_dir_all(&helper_dir_path).expect("create helper dir error");
    let resource_file_path = helper_dir_path.join("resource.ts");
    if !resource_file_path.exists() {
        info!("create default resource file");
        let mut resource_file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(resource_file_path)
            .expect("resource file open error");
        resource_file
            .write_all(
                r#"export type RequestParam = object | void;

export interface RequestConfig {
    [key:string]:string;
}

class Resource {
    post<T>(
    url: string,
    req: RequestParam,
    config?: RequestConfig
    ): Promise<any> {
        console.log("please impl post");
        return Promise.resolve();
    }

    get<T>(url: string, req: RequestParam, config?: RequestConfig): Promise<any> {
        console.log("please impl get");
        return Promise.resolve();
    }

    update<T>(
    url: string,
    req: RequestParam,
    config?: RequestConfig
    ): Promise<any> {
        console.log("please impl update");
        return Promise.resolve();
    }

    delete<T>(
    url: string,
    req: RequestParam,
    config?: RequestConfig
    ): Promise<any> {
        console.log("please impl delete");
        return Promise.resolve();
    }

    put<T>(url: string, req: RequestParam, config?: RequestConfig): Promise<any> {
        console.log("please impl put");
        return Promise.resolve();
    }
}

export const resource = new Resource();"#
                    .as_bytes(),
            )
            .expect("resource file write error");
    }
}
