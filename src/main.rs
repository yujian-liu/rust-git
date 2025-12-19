use clap::Parser;
use anyhow::Context;
use rust_git::{Cli, Commands, commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            commands::init::init().context("执行 init 命令失败")?;
        }
        Commands::Add { path } => {
            commands::add::add(&path).context(format!("执行 add 命令失败（路径：{}）", path))?;
        }
        Commands::Rm { path } => {
            commands::rm::rm(&path).context(format!("执行 rm 命令失败（路径：{}）", path))?;
        }
        Commands::Commit { message } => {
            commands::commit::commit(&message).context(format!("执行 commit 命令失败（信息：{}）", message))?;
        }
        Commands::Log => {
            commands::log::log().context("执行 log 命令失败")?;
        }
        Commands::Branch { name, delete } => {
            commands::branch::branch(name, delete).context("执行 branch 命令失败")?;
        }
        Commands::Checkout { target } => {
            commands::checkout::checkout(&target).context("执行 checkout 命令失败")?;
        }
    }

    Ok(())
}

