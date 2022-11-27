use clap::Parser;

#[derive(Debug, Clone)]
pub struct CommandConfig {
    pub workspace: String,
    pub split: bool,
    pub open_config_path: String,
    pub controller_dir_name: String,
    pub ignore_option: bool,
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
}

pub fn get_command_config() -> CommandConfig {
    let args = Args::parse();

    CommandConfig {
        workspace: args.output,
        split: args.split,
        open_config_path: args.config,
        controller_dir_name: String::from("module"),
        ignore_option: args.ignore_option
    }
}
