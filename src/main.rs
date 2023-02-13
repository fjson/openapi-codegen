mod code_gen;
mod command_config;
mod open_api;
mod open_parser;
mod tools;

use command_config::get_command_config;
use std::time::Instant;

use crate::{code_gen::ts_generator, open_parser::open_api_javascript::OpenApi3JavaScript};

#[tokio::main]
async fn main() {
    env_logger::init();

    // 获取命令行参数
    let command_config = get_command_config();

    let start = Instant::now();
    // 获取open api 配置文件内容
    let mut open_config = tools::http_request::get(&command_config.open_config_path)
        .await
        .expect("open api config get error");
    println!("get config use time: {}ms", start.elapsed().as_millis());

    // 生成 typescript open api 调用
    let start = Instant::now();
    ts_generator::create_typescript_api(&command_config, &OpenApi3JavaScript::new(&mut open_config));
    println!("generate use time: {}ms", start.elapsed().as_millis());
}
