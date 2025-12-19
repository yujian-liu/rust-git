use crate::utils::metadata;
use crate::RustGitResult;

/// 实现 git log 核心逻辑
pub fn log() -> RustGitResult<()> {
    // 检查仓库是否初始化
    if !crate::utils::fs::is_repo_initialized() {
        return Err(anyhow::anyhow!("未初始化 rust-git 仓库，请先执行 `rust-git init`"));
    }

    // 读取所有提交
    let commits = metadata::read_all_commits()?;
    if commits.is_empty() {
        println!("暂无提交记录");
        return Ok(());
    }

    // 格式化输出
    for commit in commits {
        println!("{}", metadata::format_commit(&commit));
    }

    Ok(())
}