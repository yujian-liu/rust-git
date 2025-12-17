use crate::utils::fs;
use crate::RustGitResult;

/// 实现 git init 核心逻辑
pub fn init() -> RustGitResult<()> {
    // 检查仓库是否已初始化
    if fs::is_repo_initialized() {
        println!("重新初始化已存在的 rust-git 仓库于：{}", std::env::current_dir()?.display());
        return Ok(());
    }

    // 创建仓库目录结构
    fs::create_repo_dirs()?;
    println!("初始化空的 rust-git 仓库于：{}", std::env::current_dir()?.display());

    Ok(())
}