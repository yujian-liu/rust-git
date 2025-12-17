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