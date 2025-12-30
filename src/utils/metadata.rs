use anyhow::{Context, Result};
use chrono::Local;
use chrono::TimeZone;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::utils::hash;
use crate::utils::fs as utils_fs;
use sha1::{Digest, Sha1};

/// 暂存区条目结构
#[derive(Debug, Serialize, Deserialize)]
pub struct IndexEntry {
    pub path: String,
    pub hash: String,
}

/// 提交对象结构
#[derive(Debug, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,          // 提交哈希
    pub message: String,     // 提交信息
    pub author: String,      // 作者（简化为固定值）
    pub timestamp: i64,      // 时间戳（秒）
    pub tree_hash: String,   // 目录树哈希（简化为暂存区哈希）
}

/// 生成目录树哈希（简化版：直接哈希暂存区内容）
pub fn generate_tree_hash() -> Result<String> {
    // 读取暂存区
    let index = utils_fs::read_index()?;
    let index_str = serde_json::to_string(&index)
        .context("序列化暂存区失败")?;
    
    // 计算暂存区的 SHA-1 哈希作为目录树哈希
    let mut hasher = Sha1::new();
    hasher.update(index_str.as_bytes());
    let tree_hash = format!("{:x}", hasher.finalize());

    // 存储目录树对象
    hash::store_object(&tree_hash, index_str.as_bytes())?;

    Ok(tree_hash)
}

/// 创建提交对象
pub fn create_commit(message: &str) -> Result<Commit> {
    // 生成目录树哈希
    let tree_hash = generate_tree_hash()?;
    let timestamp = Local::now().timestamp();
    
    // 构造 Git 风格的提交内容
    let commit_content = format!(
        "tree {}\nauthor RustGit <rustgit@example.com> {} +0800\ncommitter RustGit <rustgit@example.com> {} +0800\n\n{}",
        tree_hash, timestamp, timestamp, message
    );
    
    // 计算提交哈希
    let mut hasher = Sha1::new();
    hasher.update(commit_content.as_bytes());
    let commit_id = format!("{:x}", hasher.finalize());
    
    // 存储提交对象
    hash::store_object(&commit_id, commit_content.as_bytes())?;

    Ok(Commit {
        id: commit_id,
        message: message.to_string(),
        author: "RustGit <rustgit@example.com>".to_string(),
        timestamp,
        tree_hash,
    })
}

/// 保存提交记录（写入日志）
pub fn save_commit(commit: &Commit) -> Result<()> {
    // 写入提交日志
    let log_path = ".rust-git/logs/commits";
    // 创建日志目录
    if !Path::new(log_path).parent().unwrap().exists() {
        fs::create_dir_all(Path::new(log_path).parent().unwrap())?;
    }
    // 序列化提交信息
    let commit_json = serde_json::to_string_pretty(commit)
        .context("序列化提交信息失败")?;
    // 追加到日志文件
    let mut log_content = if Path::new(log_path).exists() {
        fs::read_to_string(log_path)?
    } else {
        String::new()
    };
    log_content.push_str(&format!("[{}] {}\n{}\n\n", commit.id, commit.message, commit_json));
    fs::write(log_path, log_content)
        .context("写入提交日志失败")?;

    Ok(())
}

/// 读取所有提交记录（按时间倒序）
pub fn read_all_commits() -> Result<Vec<Commit>> {
    let log_path = ".rust-git/logs/commits";
    if !Path::new(log_path).exists() {
        return Ok(Vec::new());
    }

    let log_content = fs::read_to_string(log_path)
        .context("读取提交日志失败")?;
    // 日志条目以空行分隔，保存格式为："[<id>] <message>\n<pretty JSON>\n\n"
    // 为兼容 Windows 回车，先规范化为 LF，再按两个 LF 分割条目
    let normalized = log_content.replace("\r\n", "\n");
    let mut commits = Vec::new();
    for entry in normalized.split("\n\n") {
        let entry = entry.trim();
        if entry.is_empty() {
            continue;
        }
        // 找到第一行结束位置，后续为 JSON 内容（可能多行）
        if let Some(pos) = entry.find('\n') {
            let json_part = &entry[pos + 1..];
            let commit: Commit = serde_json::from_str(json_part)
                .context("解析提交记录失败（JSON 解析错误）")?;
            commits.push(commit);
        } else {
            // 如果没有换行，跳过格式不正确的条目
            continue;
        }
    }

    // 按时间戳倒序（最新提交在前）
    commits.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(commits)
}

/// 格式化提交信息（模仿 Git log 样式）
pub fn format_commit(commit: &Commit) -> String {
    let time_dt = chrono::Local
        .timestamp_opt(commit.timestamp, 0)
        .single()
        .unwrap_or_else(|| chrono::Local::now());
    let time = time_dt.format("%Y-%m-%d %H:%M:%S %z").to_string();
    format!(
        "commit {}\nAuthor: {}\nDate:   {}\n\n    {}\n",
        commit.id, commit.author, time, commit.message
    )
}

/// 更新分支的最新提交（提交时调用）
pub fn update_branch_commit(branch_name: &str, commit_id: &str) -> Result<()> {
    crate::utils::fs::update_branch(branch_name, commit_id)
}