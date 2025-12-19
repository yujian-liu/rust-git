use anyhow::{Context, Result};
use sha1::{Digest, Sha1};
use std::fs;
use std::path::Path;

/// 计算文件内容的 SHA-1 哈希（Git 风格）
pub fn hash_file(path: &Path) -> Result<String> {
    // 读取文件内容
    let content = fs::read(path)
        .context(format!("读取文件失败：{}", path.display()))?;
    // 计算 SHA-1 哈希
    let mut hasher = Sha1::new();
    hasher.update(&content);
    // 转换为十六进制字符串
    let hash = format!("{:x}", hasher.finalize());
    Ok(hash)
}

/// 将内容存储为 Git 风格的对象（2 位目录 + 剩余哈希作为文件名）
pub fn store_object(hash: &str, content: &[u8]) -> Result<()> {
    // 拆分哈希：前 2 位为目录名，剩余为文件名（Git 标准）
    let (dir_part, file_part) = hash.split_at(2);
    let obj_dir = Path::new(".rust-git/objects").join(dir_part);
    let obj_path = obj_dir.join(file_part);

    // 创建对象目录
    if !obj_dir.exists() {
        fs::create_dir_all(&obj_dir)
            .context(format!("创建对象目录失败：{}", obj_dir.display()))?;
    }

    // 写入对象内容
    fs::write(&obj_path, content)
        .context(format!("写入对象失败：{}", obj_path.display()))?;

    Ok(())
}

/// 读取 Git 对象内容
pub fn read_object(hash: &str) -> Result<Vec<u8>> {
    let (dir_part, file_part) = hash.split_at(2);
    let obj_path = Path::new(".rust-git/objects")
        .join(dir_part)
        .join(file_part);
    
    let content = fs::read(&obj_path)
        .context(format!("读取对象失败：{}", obj_path.display()))?;
    Ok(content)
}

/// 解析提交对象，提取目录树哈希
pub fn parse_commit(commit_content: &[u8]) -> Result<String> {
    let commit_str = String::from_utf8_lossy(commit_content);
    // 提取 tree 行：tree xxxxxxxx
    let tree_line = commit_str.lines()
        .find(|line| line.starts_with("tree "))
        .ok_or_else(|| anyhow::anyhow!("提交对象无目录树信息"))?;
    let tree_hash = tree_line.trim_start_matches("tree ").trim();
    Ok(tree_hash.to_string())
}

/// 解析目录树对象，提取文件路径和哈希（简化版：暂存区内容）
pub fn parse_tree(tree_hash: &str) -> Result<serde_json::Value> {
    let tree_content = read_object(tree_hash)?;
    let tree_json = serde_json::from_slice(&tree_content)
        .context("解析目录树对象失败")?;
    Ok(tree_json)
}