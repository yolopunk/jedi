//! hosts 写入的一次性提权（Windows）。
//!
//! 普通用户令牌写 `C:\Windows\System32\drivers\etc\hosts` 会得到 os error 5（拒绝访问）。
//! 这里不要求整个应用以管理员运行，而是把新内容落到临时文件，用
//! `ShellExecuteExW` 以 `runas` 启动自身并携带 `--hosts-write <tmp>`：子进程在
//! main() 里、任何 Tauri 初始化（含 single-instance）之前完成写入即退出。
//! 用户在 UAC 上点「否」则原样返回错误，由前端走既有的回滚/通知路径。

use std::path::Path;

/// os error 5（拒绝访问）或等价的 PermissionDenied —— 触发提权重试的判据。
pub(crate) fn is_access_denied(e: &std::io::Error) -> bool {
  e.kind() == std::io::ErrorKind::PermissionDenied || e.raw_os_error() == Some(5)
}

/// 把临时文件内容覆盖到目标 hosts 文件。
///
/// hosts 偶尔会被设过只读属性（即使提权了 `fs::write` 也会失败），先清掉再写。
/// 只读位只在 Windows 语义下有意义；unix 下 `set_readonly(false)` 是无操作。
#[allow(clippy::permissions_set_readonly_false)]
fn copy_hosts(tmp: &Path, target: &Path) -> std::io::Result<()> {
  let content = std::fs::read_to_string(tmp)?;
  if target.exists() {
    let mut perms = std::fs::metadata(target)?.permissions();
    if perms.readonly() {
      perms.set_readonly(false);
      std::fs::set_permissions(target, perms)?;
    }
  }
  std::fs::write(target, content)
}

// ---------------------------------------------------------------------------
// 子进程侧：识别 `--hosts-write <tmp>` 并执行写入
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
pub fn handle_hosts_write_cli() -> Option<i32> {
  // 与 main() 里 --mcp-server 相同的早退模式：命中即处理并返回退出码。
  let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
  let idx = args.iter().position(|a| *a == "--hosts-write")?;
  let Some(tmp) = args.get(idx + 1).map(std::path::PathBuf::from) else {
    return Some(2); // 参数残缺，直接退出，不能落进 GUI 启动路径
  };

  let result = copy_hosts(&tmp, std::path::Path::new(crate::api::hosts::HOSTS_PATH));
  let _ = std::fs::remove_file(&tmp); // 无论成败都清掉交接文件
  Some(if result.is_ok() { 0 } else { 1 })
}

#[cfg(not(target_os = "windows"))]
pub fn handle_hosts_write_cli() -> Option<i32> {
  None
}

// ---------------------------------------------------------------------------
// 父进程侧：请求提权并等待子进程完成
// ---------------------------------------------------------------------------

#[cfg(target_os = "windows")]
pub fn write_elevated(content: &str) -> Result<(), String> {
  use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, ERROR_CANCELLED, WAIT_OBJECT_0, WAIT_TIMEOUT,
  };
  use windows_sys::Win32::System::Threading::{GetExitCodeProcess, WaitForSingleObject};
  use windows_sys::Win32::UI::Shell::{
    ShellExecuteExW, SEE_MASK_NOASYNC, SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW,
  };

  // 子进程是 GUI 子系统、没有 stdio，只能靠文件交接内容
  let tmp = std::env::temp_dir().join(format!("jedi-hosts-{}.tmp", uuid::Uuid::new_v4()));
  if let Err(e) = std::fs::write(&tmp, content) {
    return Err(format!("写入临时文件失败: {}", e));
  }

  let exe = match std::env::current_exe() {
    Ok(p) => p,
    Err(e) => {
      let _ = std::fs::remove_file(&tmp);
      return Err(format!("定位当前程序失败: {}", e));
    }
  };
  let verb = wide("runas");
  let file = wide(&exe.display().to_string());
  // 临时文件路径可能带空格（用户名等），必须加引号
  let parameters = wide(&format!("--hosts-write \"{}\"", tmp.display()));

  let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
  info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
  info.fMask = SEE_MASK_NOCLOSEPROCESS | SEE_MASK_NOASYNC;
  info.lpVerb = verb.as_ptr();
  info.lpFile = file.as_ptr();
  info.lpParameters = parameters.as_ptr();
  info.nShow = 0; // SW_HIDE：子进程只写文件，不进 GUI

  if unsafe { ShellExecuteExW(&mut info) } == 0 {
    let code = unsafe { GetLastError() };
    let _ = std::fs::remove_file(&tmp);
    if code == ERROR_CANCELLED {
      return Err("已取消管理员授权，hosts 未写入".to_string());
    }
    return Err(format!("请求管理员权限失败（错误码 {}）", code));
  }

  if info.hProcess.is_null() {
    // 拿不到句柄（罕见）：只能假定子进程已启动，由下一次读取校验结果
    return Ok(());
  }

  // UAC 安全桌面约 2 分钟自动消失，这里对齐放宽到 120s
  let wait = unsafe { WaitForSingleObject(info.hProcess, 120_000) };
  let mut exit_code: u32 = 1;
  unsafe { GetExitCodeProcess(info.hProcess, &mut exit_code) };
  unsafe { CloseHandle(info.hProcess) };

  match wait {
    WAIT_OBJECT_0 if exit_code == 0 => Ok(()),
    WAIT_OBJECT_0 => Err("提权进程写入 hosts 失败".to_string()),
    WAIT_TIMEOUT => Err("等待管理员写入超时".to_string()),
    _ => Err("等待提权进程异常结束".to_string()),
  }
}

#[cfg(not(target_os = "windows"))]
pub fn write_elevated(_content: &str) -> Result<(), String> {
  Err("当前平台不支持自动提权，请手动编辑 hosts 文件".to_string())
}

/// UTF-16（带 NUL 结尾），供 windows-sys 的 PCWSTR 参数使用。
#[cfg(target_os = "windows")]
fn wide(s: &str) -> Vec<u16> {
  s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn access_denied_detection() {
    assert!(is_access_denied(&std::io::Error::from_raw_os_error(5)));
    assert!(is_access_denied(&std::io::Error::new(
      std::io::ErrorKind::PermissionDenied,
      "denied"
    )));
    // 找不到文件（os error 2 / NotFound）不是权限问题，不应触发提权
    assert!(!is_access_denied(&std::io::Error::from_raw_os_error(2)));
    assert!(!is_access_denied(&std::io::Error::new(
      std::io::ErrorKind::NotFound,
      "missing"
    )));
  }

  #[test]
  fn copy_hosts_overwrites_readonly_target() {
    let dir = std::env::temp_dir().join(format!("jedi-elevate-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    let tmp = dir.join("src.tmp");
    let target = dir.join("hosts");
    std::fs::write(&tmp, "127.0.0.1 jedi.test\n").unwrap();
    std::fs::write(&target, "old content").unwrap();

    // 模拟被设过只读属性的 hosts
    let mut perms = std::fs::metadata(&target).unwrap().permissions();
    perms.set_readonly(true);
    std::fs::set_permissions(&target, perms).unwrap();

    copy_hosts(&tmp, &target).unwrap();
    assert_eq!(
      std::fs::read_to_string(&target).unwrap(),
      "127.0.0.1 jedi.test\n"
    );
    let _ = std::fs::remove_dir_all(&dir);
  }
}
