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
    Log,                      // git log：无参数（简化版）
    Branch {
        #[arg(required = false)]
        name: Option<String>, // 分支名（创建分支时必填）
        #[arg(short, long)]
        delete: Option<String>, // 删除分支（-d/--delete）
    },
    Checkout {target: String},
}

pub type RustGitResult<T> = Result<T>;

pub mod commands {
    pub mod init; 
    pub mod add;
    pub mod rm;
    pub mod commit;
    pub mod log;
    pub mod branch;
    pub mod checkout;
}

pub mod utils {
    pub mod fs; 
    pub mod hash;
    pub mod metadata;
}