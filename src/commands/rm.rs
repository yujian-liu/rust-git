use anyhow::Context;
use crate::utils::fs;
use crate::RustGitResult;
use serde_json::Value;

/// 实现 git rm 核心逻辑
pub fn rm(path: &str) -> RustGitResult<()> {
    // 检查仓库是否初始化
    if !fs::is_repo_initialized() {
        return Err(anyhow::anyhow!("未初始化 rust-git 仓库，请先执行 `rust-git init`"));
    }

    // 获取绝对路径
    let abs_path = fs::get_absolute_path(path)?;
    let repo_root = std::env::current_dir()?;
    let rel_path = abs_path.strip_prefix(&repo_root)
        .context(format!("获取相对路径失败：{}", abs_path.display()))?
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("路径转换为字符串失败：{}", abs_path.display()))?;

    // 读取暂存区
    let mut index = fs::read_index()?;
    let mut index_array = if index.is_array() {
        index.take()
    } else {
        Value::Array(Vec::new())
    };

    // 从暂存区移除条目
    let mut removed = false;
    if let Value::Array(ref mut entries) = index_array {
        entries.retain(|entry| {
            let keep = entry["path"] != rel_path;
            if !keep {
                removed = true;
            }
            keep
        });
    }

    if !removed {
        return Err(anyhow::anyhow!("文件未在暂存区中：{}", abs_path.display()));
    }

    // 删除物理文件/目录（可选，模仿 Git 的 rm 行为）
    if abs_path.exists() {
        if abs_path.is_file() {
            std::fs::remove_file(&abs_path)
                .context(format!("删除文件失败：{}", abs_path.display()))?;
        } else if abs_path.is_dir() {
            std::fs::remove_dir_all(&abs_path)
                .context(format!("删除目录失败：{}", abs_path.display()))?;
        }
    }

    // 写入更新后的暂存区
    fs::write_index(&index_array)?;
    println!("已从暂存区和文件系统移除：{}", abs_path.display());

    Ok(())
}