mod code_gen;
mod command_config;
mod open_api;
mod open_parser;
mod tools;
use crate::{code_gen::ts_generator, open_parser::open_api_javascript::OpenApi3JavaScript};
use command_config::get_command_config;

#[tokio::main]
async fn main() {
    env_logger::init();

    // 获取命令行参数
    let command_config = get_command_config();

    // 获取open api 配置文件内容
    let mut open_config = tools::http_request::get(&command_config.open_config_path)
        .await
        .expect("open api config get error");

    // 生成 typescript open api 调用
    ts_generator::create_typescript_api(
        &command_config,
        &mut OpenApi3JavaScript::new(&mut open_config, &command_config),
    );
}
