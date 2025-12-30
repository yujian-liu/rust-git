use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use winapi::um::fileapi::CreateDirectoryW;
use winapi::um::errhandlingapi::GetLastError;
use std::os::windows::ffi::OsStrExt;
use serde_json::Value;

/// 检查当前目录是否已初始化 rust-git 仓库
pub fn is_repo_initialized() -> bool {
    Path::new(".rust-git").exists()
}

/// 创建 .rust-git 目录结构
pub fn create_repo_dirs() -> Result<()> {
    let dirs = [
        ".rust-git",
        ".rust-git/objects",    // 存储对象（文件/提交/目录树）
        ".rust-git/refs",       // 引用（分支/标签）
        ".rust-git/refs/heads", // 分支存储目录
        ".rust-git/logs",       // 日志
    ];

    for dir in dirs {
        let path = Path::new(dir);
        if !path.exists() {
            let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
            let success = unsafe { CreateDirectoryW(wide_path.as_ptr(), std::ptr::null_mut()) != 0 };

            if !success {
                let err = unsafe { GetLastError() };
                if err != 183 { // 183 = 目录已存在（忽略该错误）
                    return Err(anyhow::anyhow!("创建目录失败（错误码：{}）：{}", err, dir));
                }
            }
        }
    }

    // 初始化暂存区（index）文件
    let index_path = Path::new(".rust-git/index");
    if !index_path.exists() {
        fs::write(index_path, "[]")
            .context("初始化暂存区 index 文件失败")?;
    }

    // 初始化 HEAD 文件，指向默认分支 master
    let head_path = Path::new(".rust-git/HEAD");
    if !head_path.exists() {
        fs::write(head_path, "ref: refs/heads/master")
            .context("初始化 HEAD 文件失败")?;
    }

    // 创建默认分支 master 文件
    let master_branch = Path::new(".rust-git/refs/heads/master");
    if !master_branch.exists() {
        fs::write(master_branch, "")
            .context("初始化 master 分支文件失败")?;
    }

    Ok(())
}

/// 获取文件/目录的绝对路径
pub fn get_absolute_path(path: &str) -> Result<PathBuf> {
    let path = Path::new(path);
    let abs_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .context("获取当前目录失败")?
            .join(path)
    };

    // canonicalize 之后在 Windows 上可能带有 verbatim 前缀 "\\?\\"，
    // 去除该前缀以便输出更友好（同时处理 UNC 路径的 "\\?\\UNC\\" 情况）
    let canonical = abs_path
        .canonicalize()
        .context(format!("转换为绝对路径失败：{}", path.display()))?;

    let s = canonical.to_string_lossy();
    let verbatim_unc = "\\\\?\\UNC\\";
    let verbatim = "\\\\?\\";
    let cleaned = if s.starts_with(verbatim_unc) {
        format!("\\{}", &s[verbatim_unc.len()..])
    } else if s.starts_with(verbatim) {
        s[verbatim.len()..].to_string()
    } else {
        s.to_string()
    };

    Ok(PathBuf::from(cleaned))
}

/// 读取暂存区（index）文件
pub fn read_index() -> Result<Value> {
    let index_content = fs::read_to_string(".rust-git/index")
        .context("读取暂存区 index 文件失败")?;
    let index = serde_json::from_str(&index_content)
        .context("解析 index 文件失败（JSON 格式错误）")?;
    Ok(index)
}

/// 写入暂存区（index）文件
pub fn write_index(index: &Value) -> Result<()> {
    let index_content = serde_json::to_string_pretty(index)
        .context("序列化 index 失败")?;
    fs::write(".rust-git/index", index_content)
        .context("写入 index 文件失败")?;
    Ok(())
}

/// 标准化路径分隔符（将 \ 转为 /）
pub fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// 获取 rust-git 仓库的根目录（包含 .rust-git 的目录）
pub fn get_repo_root() -> Result<PathBuf> {
    let mut current_dir = std::env::current_dir().context("获取当前目录失败")?;
    
    // 向上遍历直到找到 .rust-git 目录
    loop {
        if current_dir.join(".rust-git").exists() {
            return Ok(current_dir);
        }
        
        // 到达根目录仍未找到
        if !current_dir.pop() {
            return Err(anyhow::anyhow!("未找到 rust-git 仓库根目录（未初始化或不在仓库内）"));
        }
    }
}

/// 获取当前分支名（默认 master）
pub fn get_current_branch() -> Result<String> {
    let head_path = Path::new(".rust-git/HEAD");
    if !head_path.exists() {
        return Ok("master".to_string());
    }

    let head_content = fs::read_to_string(head_path)
        .context("读取 HEAD 失败")?;
    // HEAD 格式：ref: refs/heads/[分支名]（直接存储分支名则简化处理）
    let branch = if head_content.starts_with("ref: ") {
        head_content.trim_start_matches("ref: refs/heads/").trim().to_string()
    } else {
        // 若 HEAD 直接存储提交ID，默认 master
        "master".to_string()
    };

    Ok(branch)
}

/// 列出所有分支
pub fn list_branches() -> Result<Vec<String>> {
    let branches_dir = Path::new(".rust-git/refs/heads");
    if !branches_dir.exists() {
        return Ok(vec!["master".to_string()]);
    }

    let mut branches = Vec::new();
    for entry in fs::read_dir(branches_dir)
        .context("读取分支目录失败")?
    {
        let entry = entry.context("读取分支条目失败")?;
        if entry.file_type()?.is_file() {
            let branch_name = entry.file_name()
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("分支名转换失败"))?
                .to_string();
            branches.push(branch_name);
        }
    }

    if branches.is_empty() {
        branches.push("master".to_string());
    }

    Ok(branches)
}

/// 创建分支（关联当前 HEAD 指向的提交）
pub fn create_branch(branch_name: &str) -> Result<()> {
    // 检查分支名合法性
    if branch_name.contains('/') || branch_name.contains('\\') || branch_name.is_empty() {
        return Err(anyhow::anyhow!("分支名不合法：{}", branch_name));
    }

    // 检查分支是否已存在
    let branch_path = Path::new(".rust-git/refs/heads").join(branch_name);
    if branch_path.exists() {
        return Err(anyhow::anyhow!("分支 {} 已存在", branch_name));
    }

    // 获取当前 HEAD 指向的提交ID
    let head_path = Path::new(".rust-git/HEAD");
    let head_content = if head_path.exists() {
        fs::read_to_string(head_path)
            .context("读取 HEAD 失败")?
            .trim()
            .to_string()
    } else {
        return Err(anyhow::anyhow!("暂无提交记录，无法创建分支"));
    };

    // 分支文件存储对应提交ID
    let commit_id = if head_content.starts_with("ref: ") {
        // 若 HEAD 指向分支，读取分支对应的提交ID
        let target_branch = head_content.trim_start_matches("ref: refs/heads/").trim();
        let target_branch_path = Path::new(".rust-git/refs/heads").join(target_branch);
        fs::read_to_string(target_branch_path)
            .context(format!("读取分支 {} 失败", target_branch))?
            .trim()
            .to_string()
    } else {
        head_content
    };

    // 创建分支文件
    fs::write(&branch_path, commit_id)
        .context(format!("创建分支 {} 失败", branch_name))?;

    Ok(())
}

/// 删除分支
pub fn delete_branch(branch_name: &str) -> Result<()> {
    // 禁止删除当前分支
    let current_branch = get_current_branch()?;
    if branch_name == current_branch {
        return Err(anyhow::anyhow!("无法删除当前分支：{}", branch_name));
    }

    // 禁止删除 master 分支（可选）
    if branch_name == "master" {
        return Err(anyhow::anyhow!("禁止删除 master 分支"));
    }

    // 删除分支文件
    let branch_path = Path::new(".rust-git/refs/heads").join(branch_name);
    if !branch_path.exists() {
        return Err(anyhow::anyhow!("分支 {} 不存在", branch_name));
    }

    fs::remove_file(&branch_path)
        .context(format!("删除分支 {} 失败", branch_name))?;

    Ok(())
}

/// 更新分支指向的提交ID
pub fn update_branch(branch_name: &str, commit_id: &str) -> Result<()> {
    let branch_path = Path::new(".rust-git/refs/heads").join(branch_name);
    fs::write(&branch_path, commit_id)
        .context(format!("更新分支 {} 失败", branch_name))?;
    Ok(())
}

/// 读取分支指向的提交ID
pub fn read_branch_commit(branch_name: &str) -> Result<String> {
    let branch_path = Path::new(".rust-git/refs/heads").join(branch_name);
    if !branch_path.exists() {
        return Err(anyhow::anyhow!("分支 {} 不存在", branch_name));
    }

    let commit_id = fs::read_to_string(branch_path)
        .context(format!("读取分支 {} 失败", branch_name))?
        .trim()
        .to_string();

    Ok(commit_id)
}