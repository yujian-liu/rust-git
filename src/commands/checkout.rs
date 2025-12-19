use anyhow::Context;
use crate::utils::hash;
use crate::utils::fs as utils_fs;
use crate::RustGitResult;
use serde_json::Value;
use std::fs;

/// 实现 git checkout 核心逻辑（切换分支/恢复文件）
pub fn checkout(target: &str) -> RustGitResult<()> {
    // 检查仓库是否初始化
    if !utils_fs::is_repo_initialized() {
        return Err(anyhow::anyhow!("未初始化 rust-git 仓库，请先执行 `rust-git init`"));
    }

    // 先尝试切换分支
    let branches = utils_fs::list_branches()?;
    if branches.contains(&target.to_string()) {
        return checkout_branch(target);
    }

    // 若不是分支，尝试恢复文件
    checkout_file(target)
}

/// 切换分支
fn checkout_branch(branch_name: &str) -> RustGitResult<()> {
    // 检查分支是否存在
    let branches = utils_fs::list_branches()?;
    if !branches.contains(&branch_name.to_string()) {
        return Err(anyhow::anyhow!("分支 {} 不存在", branch_name));
    }

    // 获取当前分支
    let current_branch = utils_fs::get_current_branch()?;
    if current_branch == branch_name {
        println!("已在分支 {} 上", branch_name);
        return Ok(());
    }

    // 读取目标分支的提交ID
    let commit_id = utils_fs::read_branch_commit(branch_name)?;
    // 更新 HEAD 指向目标分支
    let head_content = format!("ref: refs/heads/{}", branch_name);
    fs::write(".rust-git/HEAD", head_content)
        .context("更新 HEAD 指向分支失败")?;

    // 从提交恢复工作区（简化版：恢复暂存区所有文件）
    restore_working_dir(&commit_id)?;

    println!("已切换到分支 {}", branch_name);
    Ok(())
}

/// 恢复文件（从最新提交/暂存区）
fn checkout_file(file_path: &str) -> RustGitResult<()> {
    // 获取绝对路径
    let abs_path = utils_fs::get_absolute_path(file_path)?;
    let repo_root_local = std::env::current_dir()?;
    let rel_path = utils_fs::normalize_path(
        abs_path.strip_prefix(&repo_root_local)?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("路径转换失败"))?
    );

    // 读取当前 HEAD 指向的提交ID
    let current_branch = utils_fs::get_current_branch()?;
    let commit_id = utils_fs::read_branch_commit(&current_branch)?;

    // 读取提交对象，获取目录树哈希
    let commit_content = hash::read_object(&commit_id)?;
    let tree_hash = hash::parse_commit(&commit_content)?;

    // 读取目录树（暂存区内容）
    let tree = hash::parse_tree(&tree_hash)?;
    let index_array = if tree.is_array() {
        tree
    } else {
        return Err(anyhow::anyhow!("目录树格式错误"));
    };

    // 查找文件条目
    let mut file_entry: Option<Value> = None;
    if let Value::Array(entries) = &index_array {
        for entry in entries {
            if entry["path"] == rel_path {
                file_entry = Some(entry.clone());
                break;
            }
        }
    }

    if file_entry.is_none() {
        return Err(anyhow::anyhow!("文件 {} 未在提交中找到", file_path));
    }

    // 读取文件对象内容并写入工作区
    let entry = file_entry.unwrap();
    let file_hash = entry["hash"].as_str()
        .ok_or_else(|| anyhow::anyhow!("文件哈希格式错误"))?;
    let file_content = hash::read_object(file_hash)?;
    fs::write(&abs_path, file_content)
        .context(format!("恢复文件 {} 失败", abs_path.display()))?;

    println!("已恢复文件：{}", abs_path.display());
    Ok(())
}

/// 从提交恢复工作区（简化版）
fn restore_working_dir(commit_id: &str) -> RustGitResult<()> {
    // 读取提交对象
    let commit_content = hash::read_object(commit_id)?;
    let tree_hash = hash::parse_commit(&commit_content)?;

    // 读取目录树（暂存区内容）
    let tree = hash::parse_tree(&tree_hash)?;
    let index_array = if tree.is_array() {
        tree
    } else {
        return Err(anyhow::anyhow!("目录树格式错误"));
    };

    // 遍历所有文件条目，恢复到工作区
    if let Value::Array(entries) = &index_array {
        for entry in entries {
            let rel_path = entry["path"].as_str()
                .ok_or_else(|| anyhow::anyhow!("文件路径格式错误"))?;
            let file_hash = entry["hash"].as_str()
                .ok_or_else(|| anyhow::anyhow!("文件哈希格式错误"))?;
            let abs_path = repo_root.join(rel_path);

            // 创建父目录
            if let Some(parent) = abs_path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)
                        .context(format!("创建目录 {} 失败", parent.display()))?;
                }
            }

            // 写入文件内容
            let file_content = hash::read_object(file_hash)?;
            fs::write(&abs_path, file_content)
                .context(format!("恢复文件 {} 失败", abs_path.display()))?;
        }
    }

    Ok(())
}

// 补充 repo_root 变量（函数内使用）
lazy_static::lazy_static! {
    static ref repo_root: std::path::PathBuf = std::env::current_dir().unwrap();
}