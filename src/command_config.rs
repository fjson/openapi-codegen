use clap::Parser;

#[derive(Debug, Clone)]
pub struct CommandConfig {
    pub workspace: String,
    pub split: bool,
    pub open_config_path: String,
    pub controller_dir_name: String,
    pub ignore_option: bool,
    pub tags: Vec<String>,
    pub prefix: String
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

    /// is aplit api file
    #[arg(short, long, default_value_t = false)]
    split: bool,

    /// ignore response required split
    #[arg(short, long, default_value_t = false)]
    ignore_option: bool,

    /// ignore response required split
    #[arg(long)]
    tags: Option<String>,

    /// prefix
    #[arg(long)]
    prefix: Option<String>,
}

pub fn get_command_config() -> CommandConfig {
    let args = Args::parse();

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
        prefix: if args.prefix.is_some() {
            format!("{}_",  args.prefix.unwrap().split_whitespace().collect::<Vec<&str>>().join("").trim().to_string())
        } else {
            String::from("")
        }
    }
}
