use anyhow::Result;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about = "Rust 实现的简易 Git", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Parser, Debug)]
pub enum Commands {
    Init,                  // 无参数
    Add { path: String },  // 接收文件/目录路径
    Rm { path: String },   // 接收文件/目录路径
    Commit { message: String }, // 接收提交信息
}

pub type RustGitResult<T> = Result<T>;

pub mod commands {
    pub mod init; 
    pub mod add;
    pub mod rm;
    pub mod commit;
}

pub mod utils {
    pub mod fs; 
    pub mod hash;
    pub mod metadata;
}