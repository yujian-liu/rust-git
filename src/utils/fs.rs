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
        ".rust-git/objects",  // 存储对象（文件/提交/目录树）
        ".rust-git/refs",     // 引用（分支/标签）
        ".rust-git/logs",     // 日志
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