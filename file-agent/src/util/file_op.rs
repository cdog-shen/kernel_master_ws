//! 文件系统原子操作层
//!
//! 收敛本 crate 的全部文件 IO，service 编排层不得直接调用 std::fs / tokio::fs。

use std::fs;
use std::io;
use std::path::Path;
use std::time::SystemTime;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// 判断路径是否存在且为普通文件
pub fn is_file(path: &Path) -> bool {
    path.is_file()
}

/// 判断路径是否存在且为目录
pub fn is_dir(path: &Path) -> bool {
    path.is_dir()
}

/// 读取整个文件内容
pub fn read_file(path: &Path) -> io::Result<Vec<u8>> {
    fs::read(path)
}

/// 删除文件
pub fn remove_file(path: &Path) -> io::Result<()> {
    fs::remove_file(path)
}

/// 递归创建目录（已存在则视为成功）
pub fn create_dir_all(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path)
}

/// 创建（或截断）文件，返回异步写句柄
pub async fn create_file(path: &Path) -> io::Result<File> {
    File::create(path).await
}

/// 向文件异步写入一个数据块
pub async fn write_chunk(file: &mut File, chunk: &[u8]) -> io::Result<()> {
    file.write_all(chunk).await
}

/// 目录条目信息
pub struct DirEntryInfo {
    pub name: String,
    pub is_dir: bool,
    pub is_file: bool,
    pub modified: SystemTime,
}

/// 读取目录下全部条目的信息
pub fn read_dir(path: &Path) -> io::Result<Vec<DirEntryInfo>> {
    let mut items = Vec::new();
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let md = entry.metadata()?;
        items.push(DirEntryInfo {
            name: entry
                .file_name()
                .into_string()
                .unwrap_or_else(|_| String::from("<invalid utf8>")),
            is_dir: md.is_dir(),
            is_file: md.is_file(),
            modified: md.modified()?,
        });
    }
    Ok(items)
}
