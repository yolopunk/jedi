use super::WindowRect;
use std::io;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{ERROR_CLASS_ALREADY_EXISTS, HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
  BeginPaint, CombineRgn, CreateRectRgn, DeleteObject, EndPaint, FillRect, GetStockObject,
  SetDCBrushColor, SetWindowRgn, DC_BRUSH, PAINTSTRUCT, RGN_DIFF, RGN_ERROR,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
  CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, SetLayeredWindowAttributes,
  SetWindowDisplayAffinity, SetWindowPos, HWND_TOPMOST, LWA_ALPHA, SWP_NOACTIVATE, SWP_NOMOVE,
  SWP_NOSIZE, SWP_SHOWWINDOW, WDA_EXCLUDEFROMCAPTURE, WM_PAINT, WNDCLASSW, WS_EX_LAYERED,
  WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
};

fn frame_bounds(
  (mx, my, mw, mh): (i32, i32, u32, u32),
  (x, y, w, h): (u32, u32, u32, u32),
  stroke: i32,
) -> WindowRect {
  // 描边优先画在选区外，贴到显示器边沿时把这一边画在选区内。
  let left = (x as i32 - stroke).max(0);
  let top = (y as i32 - stroke).max(0);
  let right = ((x + w) as i32).saturating_add(stroke).min(mw as i32);
  let bottom = ((y + h) as i32).saturating_add(stroke).min(mh as i32);
  WindowRect {
    x: mx + left,
    y: my + top,
    width: (right - left) as u32,
    height: (bottom - top) as u32,
  }
}

unsafe extern "system" fn frame_proc(
  hwnd: HWND,
  message: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  if message == WM_PAINT {
    let mut paint: PAINTSTRUCT = std::mem::zeroed();
    let dc = BeginPaint(hwnd, &mut paint);
    // COLORREF 按 BGR 排列，和 Web 录屏边框使用同一个红色（#ff5252）。
    SetDCBrushColor(dc, 0x0052_52FF);
    FillRect(dc, &paint.rcPaint, GetStockObject(DC_BRUSH));
    EndPaint(hwnd, &paint);
    return 0;
  }
  DefWindowProcW(hwnd, message, wparam, lparam)
}

unsafe fn set_frame_region(hwnd: HWND, width: i32, height: i32, stroke: i32) -> Result<(), String> {
  let outer = CreateRectRgn(0, 0, width, height);
  if outer.is_null() {
    return Err("创建录屏边框区域失败".into());
  }
  if width > 2 * stroke && height > 2 * stroke {
    let hole = CreateRectRgn(stroke, stroke, width - stroke, height - stroke);
    if hole.is_null() {
      DeleteObject(outer);
      return Err("创建录屏边框内部区域失败".into());
    }
    let combined = CombineRgn(outer, outer, hole, RGN_DIFF);
    DeleteObject(hole);
    if combined == RGN_ERROR {
      DeleteObject(outer);
      return Err("裁剪录屏边框区域失败".into());
    }
  }
  if SetWindowRgn(hwnd, outer, 0) == 0 {
    DeleteObject(outer);
    return Err("应用录屏边框区域失败".into());
  }
  // SetWindowRgn 成功后由系统管理 outer。
  Ok(())
}

/// 在主线程创建原生置顶边框。Windows 隐藏 owner 时会保留此窗，销毁 owner 时会一并销毁。
pub(super) fn show(
  owner: &tauri::WebviewWindow,
  monitor: (i32, i32, u32, u32),
  region: (u32, u32, u32, u32),
) -> Result<(), String> {
  let owner_hwnd = owner.hwnd().map_err(|e| e.to_string())?;
  let stroke = (2.0 * owner.scale_factor().unwrap_or(1.0)).round().max(2.0) as i32;
  let bounds = frame_bounds(monitor, region, stroke);
  let class_name = windows_sys::w!("JediRecordFrame");

  unsafe {
    let instance = GetModuleHandleW(null());
    if instance.is_null() {
      return Err(format!(
        "读取录屏边框窗口模块失败: {}",
        io::Error::last_os_error()
      ));
    }
    let class = WNDCLASSW {
      lpfnWndProc: Some(frame_proc),
      hInstance: instance,
      lpszClassName: class_name,
      ..std::mem::zeroed()
    };
    if RegisterClassW(&class) == 0 {
      let error = io::Error::last_os_error();
      if error.raw_os_error() != Some(ERROR_CLASS_ALREADY_EXISTS as i32) {
        return Err(format!("注册录屏边框窗口失败: {}", error));
      }
    }
    let hwnd = CreateWindowExW(
      WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
      class_name,
      null(),
      WS_POPUP,
      bounds.x,
      bounds.y,
      bounds.width as i32,
      bounds.height as i32,
      owner_hwnd.0 as _,
      null_mut(),
      instance,
      null(),
    );
    if hwnd.is_null() {
      return Err(format!(
        "创建录屏边框窗口失败: {}",
        io::Error::last_os_error()
      ));
    }
    let result = (|| {
      set_frame_region(hwnd, bounds.width as i32, bounds.height as i32, stroke)?;
      // 初始化分层窗口的显示，同时保留 WS_EX_TRANSPARENT 的鼠标穿透。
      if SetLayeredWindowAttributes(hwnd, 0, 255, LWA_ALPHA) == 0 {
        return Err(format!(
          "初始化录屏边框窗口失败: {}",
          io::Error::last_os_error()
        ));
      }
      if SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE) == 0 {
        log::warn!("无法从系统捕获排除录屏边框: {}", io::Error::last_os_error());
      }
      if SetWindowPos(
        hwnd,
        HWND_TOPMOST,
        0,
        0,
        0,
        0,
        SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
      ) == 0
      {
        return Err(format!(
          "显示录屏边框窗口失败: {}",
          io::Error::last_os_error()
        ));
      }
      Ok(())
    })();
    if result.is_err() {
      DestroyWindow(hwnd);
    }
    result
  }
}

#[cfg(test)]
mod tests {
  use super::{frame_bounds, WindowRect};

  #[test]
  fn draws_outside_an_interior_selection_on_a_secondary_monitor() {
    assert_eq!(
      frame_bounds((-1920, -100, 1920, 1080), (140, 40, 320, 200), 4),
      WindowRect {
        x: -1784,
        y: -64,
        width: 328,
        height: 208,
      }
    );
  }

  #[test]
  fn keeps_full_screen_border_inside_its_monitor() {
    assert_eq!(
      frame_bounds((-1920, -100, 1920, 1080), (0, 0, 1920, 1080), 2),
      WindowRect {
        x: -1920,
        y: -100,
        width: 1920,
        height: 1080,
      }
    );
  }

  #[test]
  fn clips_a_high_dpi_border_at_right_and_bottom_edges() {
    assert_eq!(
      frame_bounds((200, -1080, 1920, 1080), (1800, 980, 120, 100), 4),
      WindowRect {
        x: 1996,
        y: -104,
        width: 124,
        height: 104,
      }
    );
  }
}
