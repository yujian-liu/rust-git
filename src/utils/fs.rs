use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use winapi::um::fileapi::CreateDirectoryW;
use winapi::um::errhandlingapi::GetLastError;
use std::os::windows::ffi::OsStrExt;

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

    Ok(abs_path.canonicalize()
        .context(format!("转换为绝对路径失败：{}", path.display()))?)
}