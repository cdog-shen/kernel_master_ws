//! 本地进程派生原子操作（基于 std::process 的同步实现）
//!
//! 只负责"启动进程并等待结束、捕获输出"这一原子动作；
//! 退出码的语义判断（非零是否算失败）由编排层决定。

use std::process::Command;

use crate::data_structure::MailManErr;

/// 进程执行结果（stdout/stderr 为无损 UTF-8 文本）
#[derive(Debug)]
pub struct ProcessOutput {
    pub stdout: String,
    pub stderr: String,
    /// 退出码；进程被信号终止等无法获取退出码时为 -1
    pub exit_code: i32,
}

/// 同步执行外部进程，等待结束并捕获 stdout / stderr / 退出码
///
/// 仅在进程启动失败（如可执行文件不存在、权限不足）时返回 Err；
/// 进程正常执行但退出码非零仍返回 Ok，由调用方检查 `exit_code`。
pub fn run(
    program: &str,
    args: &[&str],
    workdir: Option<&str>,
) -> Result<ProcessOutput, MailManErr<'static, String>> {
    let mut cmd = Command::new(program);
    cmd.args(args);
    if let Some(dir) = workdir {
        cmd.current_dir(dir);
    }

    let output = cmd.output().map_err(|e| {
        MailManErr::new(
            500,
            "Infrastructure: process run",
            Some(format!("启动进程 {program} 失败: {e}")),
            1,
        )
    })?;

    Ok(ProcessOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}
