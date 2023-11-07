use clap::Parser;

use crate::tools::tools::capitalize;

#[derive(Debug, Clone)]
pub struct CommandConfig {
    pub workspace: String,
    pub split: bool,
    pub open_config_path: String,
    pub controller_dir_name: String,
    pub ignore_option: bool,
    pub tags: Vec<String>,
    pub opration_prefix: Option<String>,
    pub namespace: Option<String>,
}

#[derive(Parser, Debug)]
#[command(author="jason xing. <xzjhsy@gamil.com>", version, about, long_about = None)]
struct Args {
    /// output dir
    #[arg(short, long)]
    output: String,

    /// open api config url
    #[arg(short, long)]
    config: String,

    /// is split api file
    #[arg(short, long, default_value_t = false)]
    split: bool,

    /// ignore response required 
    #[arg(short, long, default_value_t = false)]
    ignore_option: bool,

    /// generate special tag api
    #[arg(long)]
    tags: Option<String>,

    /// namespace
    #[arg(long)]
    namespace: Option<String>,
}

pub fn get_command_config() -> CommandConfig {
    let args = Args::parse();

    let opration_prefix = if args.namespace.is_some() {
       args.namespace.unwrap().split_whitespace().collect::<Vec<&str>>().join("").trim().to_string()
    } else {
        String::from("")
    };


    CommandConfig {
        workspace: args.output,
        split: args.split,
        open_config_path: args.config,
        controller_dir_name: String::from("module"),
        ignore_option: args.ignore_option,
        tags: {
            if let Some(tags) = args.tags {
                tags.split(",")
                    .filter_map(|v| {
                        if v.is_empty() {
                            None
                        } else {
                            Some(v.trim().to_string())
                        }
                    })
                    .collect()
            } else {
                vec![]
            }
        },
        opration_prefix: if opration_prefix.is_empty() {
            None
        }else {
            Some(format!("{}_", &opration_prefix))
        },
        namespace: if opration_prefix.is_empty() {
            None
        }else {
            Some(capitalize(&opration_prefix).to_string())
        }
    }
}
