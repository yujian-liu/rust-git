use crate::utils::fs;
use crate::RustGitResult;

/// 实现 git branch 核心逻辑
pub fn branch(name: Option<String>, delete: Option<String>) -> RustGitResult<()> {
    // 检查仓库是否初始化
    if !fs::is_repo_initialized() {
        return Err(anyhow::anyhow!("未初始化 rust-git 仓库，请先执行 `rust-git init`"));
    }

    // 处理删除分支
    if let Some(branch_to_delete) = delete {
        fs::delete_branch(&branch_to_delete)?;
        println!("已删除分支：{}", branch_to_delete);
        return Ok(());
    }

    // 处理创建分支
    if let Some(branch_name) = name {
        fs::create_branch(&branch_name)?;
        println!("已创建分支：{}", branch_name);
        return Ok(());
    }

    // 列出所有分支
    let branches = fs::list_branches()?;
    let current_branch = fs::get_current_branch()?;
    println!("本地分支：");
    for branch in branches {
        if branch == current_branch {
            println!("* {}", branch); // 当前分支标星
        } else {
            println!("  {}", branch);
        }
    }

    Ok(())
}