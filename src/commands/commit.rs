use crate::utils::{fs, metadata};
use crate::RustGitResult;
use chrono::TimeZone;

/// 实现 git commit 核心逻辑
pub fn commit(message: &str) -> RustGitResult<()> {
    // 检查仓库是否初始化
    if !fs::is_repo_initialized() {
        return Err(anyhow::anyhow!("未初始化 rust-git 仓库，请先执行 `rust-git init`"));
    }

    // 检查暂存区是否为空
    let index = fs::read_index()?;
    if index.as_array().unwrap().is_empty() {
        return Err(anyhow::anyhow!("暂存区为空，无内容可提交"));
    }

    // 创建提交对象
    let commit = metadata::create_commit(message)?;
    
    // 保存提交记录
    metadata::save_commit(&commit)?;

    // 打印提交信息
    println!("[提交 {}] {}", commit.id, commit.message);
    println!(" 作者: {}", commit.author);
    let time = chrono::Local
        .timestamp_opt(commit.timestamp, 0)
        .single()
        .unwrap_or_else(|| chrono::Local::now());
    println!(" 时间: {}", time.format("%Y-%m-%d %H:%M:%S"));
    println!(" 目录树哈希: {}", commit.tree_hash);

    Ok(())
}