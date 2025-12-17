use anyhow::Context;
use walkdir::WalkDir;
use crate::utils::fs as utils_fs;
use crate::utils::hash;
use crate::RustGitResult;
use serde_json::Value;
use std::fs;
use std::path::Path;

/// 实现 git add 核心逻辑
pub fn add(path: &str) -> RustGitResult<()> {
    // 检查仓库是否初始化
    if !utils_fs::is_repo_initialized() {
        return Err(anyhow::anyhow!("未初始化 rust-git 仓库，请先执行 `rust-git init`"));
    }

    // 获取绝对路径并标准化
    let abs_path = utils_fs::get_absolute_path(path)?;
    if !abs_path.exists() {
        return Err(anyhow::anyhow!("文件/目录不存在：{}", abs_path.display()));
    }

    // 读取暂存区（修复核心：拆分可变借用，避免冲突）
    let mut index = utils_fs::read_index()?;
    
    // 步骤1：确保 index 是数组类型（一次性完成，无重复借用）
    if !index.is_array() {
        index = Value::Array(Vec::new());
    }
    
    // 步骤2：获取数组的可变引用（此时只有一个可变借用）
    let index_array = index.as_array_mut().unwrap();

    // 处理文件/目录
    if abs_path.is_file() {
        add_single_file(&abs_path, index_array)?;
    } else if abs_path.is_dir() {
        // 递归遍历目录下所有文件（跳过 .rust-git 目录）
        for entry in WalkDir::new(&abs_path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| !e.path().starts_with(".rust-git")) // 忽略仓库内部文件
        {
            let entry_path = entry.path();
            if entry_path.is_file() {
                add_single_file(entry_path, index_array)?;
            }
        }
    }

    // 写入更新后的暂存区
    utils_fs::write_index(&index)?;
    println!("已将 {} 添加到暂存区", abs_path.display());

    Ok(())
}

/// 添加单个文件到暂存区
fn add_single_file(file_path: &Path, index_array: &mut Vec<Value>) -> RustGitResult<()> {
    // 1. 计算文件内容的哈希值
    let file_hash = hash::hash_file(file_path)
        .context(format!("计算文件哈希失败：{}", file_path.display()))?;
    
    // 2. 将文件内容存储为 Git 对象
    let file_content = fs::read(file_path)
        .context(format!("读取文件失败：{}", file_path.display()))?;
    hash::store_object(&file_hash, &file_content)
        .context(format!("存储文件对象失败：{}", file_path.display()))?;

    // 3. 获取仓库根目录，计算相对路径（标准化分隔符）
    let repo_root = utils_fs::get_repo_root()?;
    let rel_path = file_path.strip_prefix(&repo_root)
        .context(format!(
            "文件 {} 不在 rust-git 仓库目录 {} 下",
            file_path.display(),
            repo_root.display()
        ))?
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("路径转换为字符串失败：{}", file_path.display()))?;
    let normalized_rel_path = utils_fs::normalize_path(rel_path); // 统一路径分隔符

    // 4. 更新暂存区：存在则更新哈希，不存在则新增
    let mut entry_updated = false;
    for entry in index_array.iter_mut() {
        // 匹配标准化后的路径
        if entry["path"].as_str() == Some(&normalized_rel_path) {
            entry["hash"] = Value::String(file_hash.clone());
            entry_updated = true;
            break;
        }
    }

    // 新增暂存区条目
    if !entry_updated {
        let new_entry = serde_json::json!({
            "path": normalized_rel_path,
            "hash": file_hash
        });
        index_array.push(new_entry);
    }

    Ok(())
}