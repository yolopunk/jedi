use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub(super) struct WindowRect {
  pub(super) x: i32,
  pub(super) y: i32,
  pub(super) width: u32,
  pub(super) height: u32,
}

impl WindowRect {
  /// 与显示器相交，并转成该显示器左上角为原点的坐标。
  pub(super) fn on_monitor(self, monitor: Self) -> Option<Self> {
    let left = i64::from(self.x.max(monitor.x));
    let top = i64::from(self.y.max(monitor.y));
    let right = (i64::from(self.x) + i64::from(self.width))
      .min(i64::from(monitor.x) + i64::from(monitor.width));
    let bottom = (i64::from(self.y) + i64::from(self.height))
      .min(i64::from(monitor.y) + i64::from(monitor.height));
    (right > left && bottom > top).then(|| Self {
      x: (left - i64::from(monitor.x)) as i32,
      y: (top - i64::from(monitor.y)) as i32,
      width: (right - left) as u32,
      height: (bottom - top) as u32,
    })
  }
}

/// EnumWindows 从前往后枚举；与 xcap 的 Windows 列表相比，它也包含 Jedi 自己的窗口。
#[cfg(target_os = "windows")]
pub(super) fn visible_windows() -> Vec<WindowRect> {
  use std::mem::size_of_val;
  use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
  use windows_sys::Win32::Graphics::Dwm::{
    DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS,
  };
  use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetShellWindow, GetWindow, GetWindowLongW, GetWindowTextLengthW, IsIconic,
    IsWindowVisible, GWL_EXSTYLE, GWL_STYLE, GW_OWNER, WS_EX_APPWINDOW, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_SYSMENU,
  };

  unsafe extern "system" fn collect(hwnd: HWND, data: LPARAM) -> BOOL {
    if IsWindowVisible(hwnd) == 0 || IsIconic(hwnd) != 0 || hwnd == GetShellWindow() {
      return 1;
    }
    if GetWindowTextLengthW(hwnd) == 0 {
      // Jedi 等自绘标题栏应用可以没有窗口标题；仍排除工具提示、桌面背景等浮层。
      let extended_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
      let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
      let app_window = extended_style & WS_EX_APPWINDOW != 0
        || (GetWindow(hwnd, GW_OWNER).is_null()
          && style & WS_SYSMENU != 0
          && extended_style & (WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE) == 0);
      if !app_window {
        return 1;
      }
    }
    let mut cloaked = 0u32;
    DwmGetWindowAttribute(
      hwnd,
      DWMWA_CLOAKED as u32,
      (&mut cloaked as *mut u32).cast(),
      size_of_val(&cloaked) as u32,
    );
    if cloaked != 0 {
      return 1;
    }
    let mut bounds: RECT = std::mem::zeroed();
    if DwmGetWindowAttribute(
      hwnd,
      DWMWA_EXTENDED_FRAME_BOUNDS as u32,
      (&mut bounds as *mut RECT).cast(),
      size_of_val(&bounds) as u32,
    ) >= 0
      && bounds.right > bounds.left
      && bounds.bottom > bounds.top
    {
      // EnumWindows 同步调用回调，data 指向下方仍在借用的 Vec。
      let windows = &mut *(data as *mut Vec<WindowRect>);
      windows.push(WindowRect {
        x: bounds.left,
        y: bounds.top,
        width: (bounds.right - bounds.left) as u32,
        height: (bounds.bottom - bounds.top) as u32,
      });
    }
    1
  }

  let mut windows = Vec::new();
  unsafe {
    EnumWindows(
      Some(collect),
      &mut windows as *mut Vec<WindowRect> as LPARAM,
    )
  };
  windows
}

#[cfg(not(target_os = "windows"))]
pub(super) fn visible_windows() -> Vec<WindowRect> {
  xcap::Window::all()
    .unwrap_or_default()
    .into_iter()
    .filter_map(|window| {
      if window.is_minimized().ok()? {
        return None;
      }
      if window.pid().ok()? != std::process::id() && window.title().ok()?.trim().is_empty() {
        return None;
      }
      Some(WindowRect {
        x: window.x().ok()?,
        y: window.y().ok()?,
        width: window.width().ok()?,
        height: window.height().ok()?,
      })
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::WindowRect;

  #[test]
  fn clips_spanning_window_to_negative_origin_monitor() {
    let monitor = WindowRect {
      x: -1920,
      y: -100,
      width: 1920,
      height: 1080,
    };
    let window = WindowRect {
      x: -200,
      y: -130,
      width: 800,
      height: 500,
    };
    assert_eq!(
      window.on_monitor(monitor),
      Some(WindowRect {
        x: 1720,
        y: 0,
        width: 200,
        height: 470
      })
    );
    assert_eq!(WindowRect { x: 0, ..window }.on_monitor(monitor), None);
  }
}
