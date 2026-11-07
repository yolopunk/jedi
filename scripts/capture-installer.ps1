<#
.SYNOPSIS
    Drive a (preview) NSIS installer / uninstaller wizard and screenshot every page.

.DESCRIPTION
    Launches the given .exe, pins the wizard window to the top, walks it page by page
    with Enter (auto-clicking the forward button as a fallback) and saves one PNG per
    page.  Pages are detected structurally - by reading the texts of the dialog's child
    controls - so the progress page is captured exactly once, no matter how long the
    installation animation keeps painting.

    Designed for scripts/preview-installer.mjs output, whose Install/Uninstall sections
    are no-ops: it is therefore safe to press Enter on the options/finish page
    (nothing is installed, no app is launched, no shortcut is created).
    The process tree is ALWAYS killed at the end.

    Page file names:  <OutDir>/<Prefix>-<Lang>-<NN>-<label>.png

.PARAMETER Exe
    Preview installer (or its preview uninstall.exe).

.PARAMETER OutDir
    Screenshot directory (created when missing). Default: .tmp/installer-shots

.PARAMETER Prefix
    File name prefix, e.g. `before` / `after` / `uninstall`.

.PARAMETER Lang
    Language tag written into the file name (zh / en / ...).

.PARAMETER PageLabels
    Wizard mode only. Comma separated labels for page 1..N, e.g. "welcome,options,directory,progress,finish".
    Missing labels fall back to `page-NN`.

.PARAMETER States
    State mode for the single-window installer (no MUI wizard pages). Comma separated
    state names, e.g. "idle,progress,done". One PNG per state, named
    `<Prefix>-<Lang>-<state>.png`. Use -Actions to describe how to reach each state.

.PARAMETER Actions
    State mode only: comma separated actions, one per state transition (States.Count - 1).
      enter                 post/keyboard Enter
      click:X,Y             left click at window-relative pixel (as seen in the PNG)
      enter-or-click:X,Y    Enter first, click only if the frame did not change
      wait:MS               just wait
      none                  do nothing

.PARAMETER StateWaitMs
    Settle time after an action before the state is captured (default 400 ms).

.PARAMETER SafetyLog
    When set, a machine-state snapshot (HKCU uninstall key, D:\softwares\Jedi mtimes,
    %LOCALAPPDATA%\Jedi) is appended to this file before and after the run.

.EXAMPLE
    pwsh -File scripts/capture-installer.ps1 -Exe .tmp/installer-preview/after-zh/nsis-output.exe `
         -Prefix after -Lang zh -States idle,progress,done -Actions enter-or-click:380,430,wait:800

.EXAMPLE
    pwsh -File scripts/capture-installer.ps1 -Exe .tmp/installer-preview/before-zh/nsis-output.exe `
         -Prefix before -Lang zh -PageLabels welcome,directory,progress,finish `
         -SafetyLog .tmp/installer-shots/safety-check.txt
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Exe,
    [string]$OutDir = '.tmp/installer-shots',
    [string]$Prefix = 'shot',
    [string]$Lang = 'zh',
    [string]$TitleContains = '',
    [int]$Pages = 8,
    [int]$StepDelayMs = 300,
    [int]$LaunchTimeoutMs = 25000,
    [int]$WaitPageMs = 12000,
    [int]$MaxStall = 3,
    [string]$PageLabels = '',
    [string]$States = '',
    [string]$Actions = '',
    [int]$StateWaitMs = 400,
    [switch]$NoRun,
    [string]$SafetyLog = '',
    [switch]$KeepWindow,
    [string]$SnapshotOnly = ''
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version 2.0

# --------------------------------------------------------------------------- #
# Win32 helpers (DPI aware; CopyFromScreen based like the verified prototype)
# --------------------------------------------------------------------------- #
$cs = @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;

public static class JediCap
{
    public struct RECT { public int Left, Top, Right, Bottom; }
    public delegate bool EnumProc(IntPtr hWnd, IntPtr lParam);

    [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc cb, IntPtr p);
    [DllImport("user32.dll")] static extern bool EnumChildWindows(IntPtr parent, EnumProc cb, IntPtr p);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowTextW(IntPtr h, StringBuilder sb, int max);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassNameW(IntPtr h, StringBuilder sb, int max);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
    [DllImport("user32.dll")] static extern bool IsWindowEnabled(IntPtr h);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
    [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] static extern bool BringWindowToTop(IntPtr h);
    [DllImport("user32.dll")] static extern bool ShowWindow(IntPtr h, int cmd);
    [DllImport("user32.dll")] static extern bool SetWindowPos(IntPtr h, IntPtr after, int x, int y, int cx, int cy, uint flags);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern IntPtr SendMessageW(IntPtr h, uint msg, IntPtr w, IntPtr l);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern bool PostMessageW(IntPtr h, uint msg, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] static extern void keybd_event(byte bVk, byte bScan, uint dwFlags, UIntPtr extra);
    [DllImport("user32.dll")] static extern bool SetProcessDpiAwarenessContext(IntPtr v);
    [DllImport("user32.dll")] static extern bool SetProcessDPIAware();
    [DllImport("user32.dll")] static extern uint GetDpiForSystem();
    [DllImport("user32.dll")] static extern IntPtr GetThreadDpiAwarenessContext();
    [DllImport("user32.dll")] static extern int GetAwarenessFromDpiAwarenessContext(IntPtr c);
    [DllImport("user32.dll")] static extern IntPtr SetThreadDpiAwarenessContext(IntPtr v);

    /// <summary>
    /// Make this process per-monitor-v2 DPI aware so GetWindowRect/BitBlt work in
    /// physical pixels (the machine runs at 150%: a DPI-unaware caller would see a
    /// 751x540 window as 503x362 and capture the wrong screen region).
    /// </summary>
    public static string MakeDpiAware()
    {
        var sb = new System.Text.StringBuilder();
        bool ok = false;
        try { ok = SetProcessDpiAwarenessContext(new IntPtr(-4)); } catch { }
        sb.Append("process-dpi-aware=").Append(ok);
        if (!ok) { try { sb.Append(" classic=").Append(SetProcessDPIAware()); } catch { } }
        try
        {
            // per-thread context works even when the process context was already locked
            IntPtr prev = SetThreadDpiAwarenessContext(new IntPtr(-4));
            sb.Append(" thread-prev=").Append(prev);
        }
        catch { }
        sb.Append(" sys-dpi=").Append(GetDpiForSystem());
        try { sb.Append(" awareness=").Append(GetAwarenessFromDpiAwarenessContext(GetThreadDpiAwarenessContext())); } catch { }
        return sb.ToString();
    }
    [DllImport("user32.dll")] static extern bool IsWindow(IntPtr h);

    public const uint WM_KEYDOWN = 0x0100;
    public const uint WM_KEYUP = 0x0101;
    public const uint BM_CLICK = 0x00F5;


    public static string Title(IntPtr h)
    {
        var sb = new StringBuilder(1024);
        GetWindowTextW(h, sb, sb.Capacity);
        return sb.ToString();
    }

    public static string ClassOf(IntPtr h)
    {
        var sb = new StringBuilder(256);
        GetClassNameW(h, sb, sb.Capacity);
        return sb.ToString();
    }

    public static uint PidOf(IntPtr h)
    {
        uint pid;
        GetWindowThreadProcessId(h, out pid);
        return pid;
    }

    public static List<IntPtr> TopWindows()
    {
        var list = new List<IntPtr>();
        // no title filter: the single-window redesign may set a custom/blank caption,
        // and the caller already filters by owning process + geometry
        EnumWindows((h, p) => { if (IsWindowVisible(h)) list.Add(h); return true; }, IntPtr.Zero);
        return list;
    }

    public static List<IntPtr> Children(IntPtr parent)
    {
        var list = new List<IntPtr>();
        EnumChildWindows(parent, (h, p) => { list.Add(h); return true; }, IntPtr.Zero);
        return list;
    }

    /// <summary>Structural page fingerprint: class|text of every visible child carrying text.</summary>
    public static string Identity(IntPtr hwnd)
    {
        var parts = new List<string>();
        foreach (var c in Children(hwnd))
        {
            if (!IsWindowVisible(c)) continue;
            string t = Title(c).Trim();
            if (t.Length == 0) continue;
            string cls = ClassOf(c);
            if (cls == "msctls_progress32") continue;
            parts.Add(cls + "|" + t.Replace("&", ""));
        }
        parts.Sort(StringComparer.Ordinal);
        return string.Join("\n", parts);
    }

    static Dictionary<string, string> ChildTextSnapshot(IntPtr hwnd)
    {
        var d = new Dictionary<string, string>();
        foreach (var c in Children(hwnd))
        {
            if (!IsWindowVisible(c)) continue;
            string t = Title(c).Trim();
            if (t.Length == 0) continue;
            string cls = ClassOf(c);
            if (cls == "msctls_progress32") continue;
            d[c.ToInt64().ToString()] = cls + "|" + t.Replace("&", "");
        }
        return d;
    }

    /// <summary>
    /// Page fingerprint that ignores volatile controls: the install page repaints its
    /// "current step" static for every DetailPrint, which would otherwise look like a
    /// page change. Two snapshots <paramref name="gapMs"/> apart are intersected, so
    /// only controls whose text stayed the same count.
    /// </summary>
    public static string StableIdentity(IntPtr hwnd, int gapMs)
    {
        var a = ChildTextSnapshot(hwnd);
        System.Threading.Thread.Sleep(gapMs);
        var b = ChildTextSnapshot(hwnd);
        var stable = new List<string>();
        foreach (var kv in a)
        {
            string v;
            if (b.TryGetValue(kv.Key, out v) && v == kv.Value) stable.Add(kv.Value);
        }
        stable.Sort(StringComparer.Ordinal);
        return string.Join("\n", stable);
    }

    static readonly string[] ForwardLabels = new string[] {
        "next", "next >", "install", "finish", "close", "uninstall", "yes", "ok",
        "下一步", "下一步(n)", "安装", "完成", "关闭", "卸载", "是", "确定",
        "weiter", "installieren", "fertig", "schließen"
    };

    public static IntPtr ForwardButton(IntPtr hwnd)
    {
        foreach (var c in Children(hwnd))
        {
            if (!IsWindowVisible(c)) continue;
            if (ClassOf(c) != "Button") continue;
            string t = Title(c).Trim().Replace("&", "").ToLowerInvariant();
            if (t.Length == 0) continue;
            foreach (var lbl in ForwardLabels)
                if (t == lbl || t == lbl + "(n)" || t.StartsWith(lbl + "("))
                    return c;
        }
        return IntPtr.Zero;
    }

    public static bool Enabled(IntPtr h) { return IsWindowEnabled(h); }
    public static bool Alive(IntPtr h) { return IsWindow(h); }
    public static bool Visible(IntPtr h) { return IsWindowVisible(h); }

    public static string ForwardLabel(IntPtr hwnd)
    {
        IntPtr b = ForwardButton(hwnd);
        return b == IntPtr.Zero ? "" : Title(b).Replace("&", "").Trim();
    }

    /// <summary>True when the forward button is Finish/Close, i.e. the last page of the wizard.</summary>
    public static bool IsTerminalForward(IntPtr hwnd)
    {
        string t = ForwardLabel(hwnd).ToLowerInvariant();
        foreach (var l in new string[] { "finish", "close", "fertig", "schließen", "schliessen", "完成", "关闭", "關閉" })
            if (t.StartsWith(l)) return true;
        return false;
    }

    /// <summary>
    /// Only Next/Install may ever be auto-clicked. Clicking anything else (Finish,
    /// Close, Cancel) could close the wizard or trigger the finish-page actions.
    /// </summary>
    public static bool IsClickableForward(IntPtr hwnd)
    {
        string t = ForwardLabel(hwnd).ToLowerInvariant();
        foreach (var l in new string[] { "next", "install", "weiter", "installieren", "下一步", "安装" })
            if (t.StartsWith(l)) return true;
        return false;
    }

    public static void Focus(IntPtr h)
    {
        ShowWindow(h, 9);                 // SW_RESTORE (un-minimize, activate)
        SetWindowPos(h, new IntPtr(-1), 60, 60, 0, 0, 0x0001 | 0x0002 | 0x0040); // NOSIZE|NOMOVE|SHOWWINDOW, TOPMOST
        BringWindowToTop(h);
        SetForegroundWindow(h);
    }

    public static void SendEnter()
    {
        keybd_event(0x0D, 0, 0, UIntPtr.Zero);
        System.Threading.Thread.Sleep(40);
        keybd_event(0x0D, 0, 2, UIntPtr.Zero);
    }

    public static void PostEnter(IntPtr h)
    {
        PostMessageW(h, WM_KEYDOWN, new IntPtr(0x0D), IntPtr.Zero);
        System.Threading.Thread.Sleep(40);
        PostMessageW(h, WM_KEYUP, new IntPtr(0x0D), IntPtr.Zero);
    }

    public static void Click(IntPtr h) { SendMessageW(h, BM_CLICK, IntPtr.Zero, IntPtr.Zero); }

    public struct POINT { public int X, Y; }
    [DllImport("user32.dll")] static extern bool SetCursorPos(int x, int y);
    [DllImport("user32.dll")] static extern bool GetCursorPos(out POINT p);
    [DllImport("user32.dll")] static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);

    /// <summary>Real left click at absolute screen coordinates (works for owner-drawn controls).</summary>
    public static void ClickScreen(int x, int y)
    {
        POINT old;
        GetCursorPos(out old);
        SetCursorPos(x, y);
        System.Threading.Thread.Sleep(60);
        mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero);   // LEFTDOWN
        System.Threading.Thread.Sleep(50);
        mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);   // LEFTUP
        System.Threading.Thread.Sleep(60);
        SetCursorPos(old.X, old.Y);
    }

    [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
    public static bool IsForeground(IntPtr h) { return GetForegroundWindow() == h; }

    /// <summary>
    /// Find a visible child (Button/Static/...) by caption. An exact caption match wins
    /// over a substring match, so "卸载" hits the button and not the headline
    /// "卸载 Jedi 工具箱".
    /// </summary>
    public static IntPtr ChildByText(IntPtr parent, string needle)
    {
        IntPtr partial = IntPtr.Zero;
        foreach (var c in Children(parent))
        {
            if (!IsWindowVisible(c)) continue;
            string t = Title(c).Trim();
            if (t.Length == 0) continue;
            if (string.Equals(t, needle, StringComparison.OrdinalIgnoreCase)) return c;
            if (partial == IntPtr.Zero && t.IndexOf(needle, StringComparison.OrdinalIgnoreCase) >= 0) partial = c;
        }
        return partial;
    }

    public static List<string> VisibleChildTexts(IntPtr parent)
    {
        var list = new List<string>();
        foreach (var c in Children(parent))
            if (IsWindowVisible(c)) { string t = Title(c).Trim(); if (t.Length > 0) list.Add(t); }
        return list;
    }

    /// <summary>
    /// Post a left click straight to a control HWND. Needed for owner-drawn "static
    /// buttons" whose window rect is not where they are painted (or is bogus).
    /// </summary>
    public static void PressChild(IntPtr child)
    {
        IntPtr lp = new IntPtr((5 << 16) | 5);
        PostMessageW(child, 0x0201, new IntPtr(1), lp);   // WM_LBUTTONDOWN
        System.Threading.Thread.Sleep(60);
        PostMessageW(child, 0x0202, IntPtr.Zero, lp);    // WM_LBUTTONUP
    }

    /// <summary>Window bounds as [left, top, width, height].</summary>
    public static int[] Bounds(IntPtr hwnd)
    {
        RECT r;
        if (!GetWindowRect(hwnd, out r)) return new int[] { 0, 0, 0, 0 };
        return new int[] { r.Left, r.Top, r.Right - r.Left, r.Bottom - r.Top };
    }

    public static string Rect(IntPtr hwnd)
    {
        int[] b = Bounds(hwnd);
        return string.Format("{0},{1} {2}x{3}", b[0], b[1], b[2], b[3]);
    }
}

public static class JediGdi
{
    [DllImport("user32.dll")] public static extern IntPtr GetDC(IntPtr h);
    [DllImport("user32.dll")] public static extern int ReleaseDC(IntPtr h, IntPtr dc);
    [DllImport("gdi32.dll")] public static extern IntPtr CreateCompatibleDC(IntPtr dc);
    [DllImport("gdi32.dll")] public static extern IntPtr CreateCompatibleBitmap(IntPtr dc, int w, int h);
    [DllImport("gdi32.dll")] public static extern IntPtr SelectObject(IntPtr dc, IntPtr o);
    [DllImport("gdi32.dll")] public static extern bool BitBlt(IntPtr dst, int x, int y, int w, int h, IntPtr src, int sx, int sy, int rop);
    [DllImport("gdi32.dll")] public static extern bool DeleteObject(IntPtr o);
    [DllImport("gdi32.dll")] public static extern bool DeleteDC(IntPtr dc);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr hwnd, IntPtr hdc, uint flags);

    public const int SRCCOPY = 0x00CC0020;
    public const int CAPTUREBLT = 0x40000000;

    /// <summary>Copy a screen region into an HBITMAP (caller must convert + DeleteObject).</summary>
    public static IntPtr GrabBitmap(int x, int y, int w, int h)
    {
        IntPtr screen = GetDC(IntPtr.Zero);
        IntPtr mem = CreateCompatibleDC(screen);
        IntPtr hbmp = CreateCompatibleBitmap(screen, w, h);
        IntPtr old = SelectObject(mem, hbmp);
        BitBlt(mem, 0, 0, w, h, screen, x, y, SRCCOPY | CAPTUREBLT);
        SelectObject(mem, old);
        DeleteDC(mem);
        ReleaseDC(IntPtr.Zero, screen);
        return hbmp;
    }

    /// <summary>Ask the window to render itself into an HBITMAP (works when occluded).</summary>
    public static IntPtr PrintWindowBitmap(IntPtr hwnd, int w, int h)
    {
        IntPtr screen = GetDC(IntPtr.Zero);
        IntPtr mem = CreateCompatibleDC(screen);
        IntPtr hbmp = CreateCompatibleBitmap(screen, w, h);
        IntPtr old = SelectObject(mem, hbmp);
        PrintWindow(hwnd, mem, 2); // PW_RENDERFULLCONTENT
        SelectObject(mem, old);
        DeleteDC(mem);
        ReleaseDC(IntPtr.Zero, screen);
        return hbmp;
    }

    public static void FreeBitmap(IntPtr h) { if (h != IntPtr.Zero) DeleteObject(h); }
}
'@

Add-Type -AssemblyName System.Drawing | Out-Null
if (-not ('JediCap' -as [type])) { Add-Type -TypeDefinition $cs -Language CSharp | Out-Null }
$script:DpiInfo = [JediCap]::MakeDpiAware()

function Get-BitmapNonBlackRatio {
    param($Bitmap, [int]$Step = 17)
    $nonBlack = 0; $sampled = 0
    $w = $Bitmap.Width; $h = $Bitmap.Height
    for ($y = 0; $y -lt $h; $y += $Step) {
        for ($x = 0; $x -lt $w; $x += $Step) {
            $sampled++
            $c = $Bitmap.GetPixel($x, $y)
            if (($c.R + $c.G + $c.B) -gt 24) { $nonBlack++ }
        }
    }
    if ($sampled -eq 0) { return 0 }
    return $nonBlack / $sampled
}

function Save-WindowShot {
    param([IntPtr]$Hwnd, [string]$File)
    $b = [JediCap]::Bounds($Hwnd)
    $w = $b[2]; $h = $b[3]
    if ($w -le 0 -or $h -le 0) { return 'empty window rect' }

    $note = $null
    $hb = [JediGdi]::GrabBitmap($b[0], $b[1], $w, $h)
    $bmp = $null
    try {
        if ($hb -ne [IntPtr]::Zero) { $bmp = [System.Drawing.Image]::FromHbitmap($hb) }
        $ratio = if ($bmp) { Get-BitmapNonBlackRatio -Bitmap $bmp } else { 0 }
        if (-not $bmp -or $ratio -lt 0.01) {
            # screen copy unusable (occluded / secured desktop / minimized) -> let the window paint itself
            $screenRatio = $ratio
            if ($bmp) { $bmp.Dispose(); $bmp = $null }
            [JediGdi]::FreeBitmap($hb) | Out-Null
            $hb = [JediGdi]::PrintWindowBitmap($Hwnd, $w, $h)
            $bmp = [System.Drawing.Image]::FromHbitmap($hb)
            $ratio = Get-BitmapNonBlackRatio -Bitmap $bmp
            $note = "watch: screen copy was blank (nonblack=$([math]::Round($screenRatio, 3))), used PrintWindow"
            if ($ratio -lt 0.01) { $note = 'BLACK FRAME (window minimised, occluded or session locked)' }
        }
        $bmp.Save($File, [System.Drawing.Imaging.ImageFormat]::Png)
    } finally {
        if ($bmp) { $bmp.Dispose() }
        [JediGdi]::FreeBitmap($hb) | Out-Null
    }
    return $note
}

# --------------------------------------------------------------------------- #
# helpers
# --------------------------------------------------------------------------- #
$script:LogFile = $null
function Write-Log {
    param([string]$Msg)
    Write-Host $Msg
    if ($script:LogFile) { Add-Content -LiteralPath $script:LogFile -Value $Msg -Encoding utf8 }
}

function Get-SafetySnapshot {
    param([string]$Label)
    $sb = New-Object System.Text.StringBuilder
    [void]$sb.AppendLine("### [$Label] $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')")
    try {
        $k = Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\Jedi' -ErrorAction Stop
        foreach ($p in 'DisplayName', 'DisplayVersion', 'Publisher', 'InstallLocation', 'UninstallString', 'EstimatedSize', 'MainBinaryName') {
            [void]$sb.AppendLine("  HKCU\...\Uninstall\Jedi :: $p = $($k.$p)")
        }
    } catch {
        [void]$sb.AppendLine('  HKCU\...\Uninstall\Jedi :: <absent>')
    }
    try {
        $m = Get-ItemProperty 'HKCU:\Software\jedi\Jedi' -ErrorAction Stop
        [void]$sb.AppendLine("  HKCU\Software\jedi\Jedi = $($m.'(default)')")
    } catch {
        [void]$sb.AppendLine('  HKCU\Software\jedi\Jedi :: <absent>')
    }
    $runKey = Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run' -ErrorAction SilentlyContinue
    $run = $null
    if ($runKey -and $runKey.PSObject.Properties.Match('Jedi').Count -gt 0) { $run = $runKey.Jedi }
    [void]$sb.AppendLine("  HKCU Run\Jedi = $(if ($null -eq $run) { '<absent>' } else { $run })")

    if (Test-Path 'D:\softwares\Jedi') {
        $files = @(Get-ChildItem 'D:\softwares\Jedi' -Recurse -File -ErrorAction SilentlyContinue)
        $newest = $files | Sort-Object LastWriteTime -Descending | Select-Object -First 1
        if ($newest) {
            [void]$sb.AppendLine("  D:\softwares\Jedi :: exists, files=$($files.Count), newest=$($newest.FullName) @ $($newest.LastWriteTime.ToString('yyyy-MM-dd HH:mm:ss'))")
        } else {
            [void]$sb.AppendLine("  D:\softwares\Jedi :: exists, files=0")
        }
        [void]$sb.AppendLine("  D:\softwares\Jedi :: dir mtime=$((Get-Item 'D:\softwares\Jedi').LastWriteTime.ToString('yyyy-MM-dd HH:mm:ss'))")
    } else {
        [void]$sb.AppendLine('  D:\softwares\Jedi :: <absent>')
    }
    $local = Join-Path $env:LOCALAPPDATA 'Jedi'
    if (Test-Path $local) {
        [void]$sb.AppendLine("  LOCALAPPDATA\Jedi :: exists, mtime=$((Get-Item $local).LastWriteTime.ToString('yyyy-MM-dd HH:mm:ss')), entries=$((Get-ChildItem $local -ErrorAction SilentlyContinue | Measure-Object).Count)")
    } else {
        [void]$sb.AppendLine('  LOCALAPPDATA\Jedi :: <absent>')
    }
    $appdata = Join-Path $env:APPDATA 'io.jedi'
    [void]$sb.AppendLine("  APPDATA\io.jedi :: $(if (Test-Path $appdata) { 'exists' } else { '<absent>' })")
    foreach ($lnk in @((Join-Path ([Environment]::GetFolderPath('Desktop')) 'Jedi.lnk'),
                       (Join-Path ([Environment]::GetFolderPath('Programs')) 'Jedi.lnk'))) {
        [void]$sb.AppendLine("  shortcut $([IO.Path]::GetFileName($lnk)) :: $(if (Test-Path $lnk) { 'exists' } else { '<absent>' })")
    }
    $procs = @(Get-Process -Name 'nsis-output', 'uninstall', 'Jedi', 'Au_', 'Un_A' -ErrorAction SilentlyContinue |
        Select-Object -ExpandProperty ProcessName)
    [void]$sb.AppendLine("  processes :: $(if ($procs.Count -gt 0) { ($procs -join ',') } else { '<none>' })")
    $sb.ToString()
}

function Write-Snapshot {
    param([string]$Label)
    $text = Get-SafetySnapshot -Label $Label
    if ($SafetyLog) {
        $dir = Split-Path -Parent $SafetyLog
        if ($dir -and -not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
        Add-Content -LiteralPath $SafetyLog -Value $text -Encoding utf8
    }
    Write-Host $text
}

if ($SnapshotOnly) {
    Write-Snapshot -Label $SnapshotOnly
    return
}

$labels = @()
if ($PageLabels) { $labels = $PageLabels.Split(',') | ForEach-Object { $_.Trim() } }

# --------------------------------------------------------------------------- #
# -NoRun: validate everything, launch nothing
# --------------------------------------------------------------------------- #
if ($NoRun) {
    $problems = @()
    $exePath = $null
    try { $exePath = (Resolve-Path -LiteralPath $Exe -ErrorAction Stop).Path } catch { $problems += "exe not found: $Exe" }
    if ($exePath -and -not (Test-Path -LiteralPath $exePath -PathType Leaf)) { $problems += "exe is not a file: $exePath" }
    if (-not ('JediCap' -as [type])) { $problems += 'Win32 helper type JediCap missing (Add-Type failed)' }
    if (-not ('JediGdi' -as [type])) { $problems += 'GDI helper type JediGdi missing (Add-Type failed)' }
    try {
        if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Force -Path $OutDir | Out-Null }
        $probe = Join-Path $OutDir '.write-probe.tmp'
        Set-Content -LiteralPath $probe -Value 'ok' -Encoding utf8
        Remove-Item -LiteralPath $probe -Force
    } catch { $problems += "output directory is not writable: $OutDir" }

    # state/action syntax
    if ($States) {
        $n = @($States.Split(',') | Where-Object { $_.Trim() }).Count
        $acts = @()
        if ($Actions) { $acts = @($Actions.Split($(if ($Actions.Contains(';')) { ';' } else { ',' })) | Where-Object { $_.Trim() }) }
        if ($n -gt 1 -and $acts.Count -lt ($n - 1)) { $problems += "States needs $($n - 1) action(s), got $($acts.Count)" }
        foreach ($a in $acts) {
            if ($a -match '^(enter|none)$' -or $a -match '^wait:\d+$' -or $a -match '^(click|enter-or-click):-?\d+,-?\d+$' -or $a -match '^press:.+$') { continue }
            $problems += "unparseable action: '$a'"
        }
    } elseif (-not $PageLabels) {
        Write-Host 'note: neither -States nor -PageLabels given (labels fall back to page-NN)' -ForegroundColor Yellow
    }

    Write-Host ''
    Write-Host '================ capture-installer self-check (-NoRun) ================' -ForegroundColor Cyan
    Write-Host "exe            : $(if ($exePath) { $exePath } else { $Exe })"
    Write-Host "outdir         : $OutDir"
    Write-Host "prefix / lang  : $Prefix / $Lang"
    Write-Host "dpi            : $script:DpiInfo"
    Write-Host "mode           : $(if ($States) { "states [$States] actions [$(if ($Actions) { $Actions } else { 'enter' })]" } else { "wizard pages, up to $Pages, labels [$PageLabels]" })"
    Write-Host "title filter   : $(if ($TitleContains) { "'$TitleContains'" } else { '<any window owned by the launched process>' })"
    Write-Host "safety log     : $(if ($SafetyLog) { $SafetyLog } else { '<off>' })"
    Write-Host "helpers        : $(if (('JediCap' -as [type]) -and ('JediGdi' -as [type])) { 'JediCap + JediGdi loaded' } else { 'MISSING' })"
    $vis = @([JediCap]::TopWindows().Count)
    Write-Host "visible windows now: $($vis[0]) (nothing was launched)"
    if ($problems.Count -gt 0) {
        Write-Host ''
        Write-Host 'SELF-CHECK FAILED:' -ForegroundColor Red
        $problems | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
        exit 3
    }
    Write-Host ''
    Write-Host 'SELF-CHECK PASSED - no process was started' -ForegroundColor Green
    return
}

# --------------------------------------------------------------------------- #
# resolve paths, prepare output
# --------------------------------------------------------------------------- #
$Exe = (Resolve-Path -LiteralPath $Exe).Path
if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Force -Path $OutDir | Out-Null }
$OutDir = (Resolve-Path -LiteralPath $OutDir).Path
$script:LogFile = Join-Path $OutDir "$Prefix-$Lang-capture.log"
Set-Content -LiteralPath $script:LogFile -Value "capture $Prefix ($Lang) <- $Exe" -Encoding utf8

Write-Log "exe      : $Exe"
Write-Log "outdir   : $OutDir"
Write-Log "dpi      : $script:DpiInfo"
if ($States) {
    Write-Log "mode     : states [$States]"
} else {
    Write-Log "pages    : up to $Pages, title ~ '$TitleContains'"
}

if ($SafetyLog) { Write-Snapshot -Label "before-run $Prefix/$Lang" }

# --------------------------------------------------------------------------- #
# launch
# --------------------------------------------------------------------------- #
$launchedAt = Get-Date
$proc = Start-Process -FilePath $Exe -PassThru
Write-Log "started  : pid $($proc.Id)"

function Get-ProcessTreePids {
    param([int]$RootPid)
    $all = Get-CimInstance Win32_Process -Property ProcessId, ParentProcessId -ErrorAction SilentlyContinue
    $set = New-Object System.Collections.Generic.List[int]
    $set.Add($RootPid)
    $changed = $true
    while ($changed) {
        $changed = $false
        foreach ($p in $all) {
            if ($set.Contains([int]$p.ParentProcessId) -and -not $set.Contains([int]$p.ProcessId)) {
                $set.Add([int]$p.ProcessId)
                $changed = $true
            }
        }
    }
    , $set
}

function Find-WizardWindow {
    param([int]$RootPid, [string]$TitlePart, [switch]$AllowForeign)
    $tree = Get-ProcessTreePids -RootPid $RootPid
    $best = [IntPtr]::Zero
    $bestScore = -1
    foreach ($h in [JediCap]::TopWindows()) {
        $t = [JediCap]::Title($h)
        if ($TitlePart -and $t -notlike "*$TitlePart*") { continue }
        $pid2 = [int][JediCap]::PidOf($h)
        $inTree = $tree.Contains($pid2)
        if (-not $inTree -and -not $AllowForeign) { continue }   # never grab a window from another run
        $b = [JediCap]::Bounds($h)
        if ($b[2] -lt 200 -or $b[3] -lt 150) { continue }        # skip tooltips / IME helpers
        $score = 100
        if ([JediCap]::ClassOf($h) -eq '#32770') { $score += 50 }
        if ($t.Length -gt 0) { $score += 5 }
        if ($score -gt $bestScore) { $bestScore = $score; $best = $h }
    }
    if ($bestScore -le 0) { return [IntPtr]::Zero }
    return $best
}

<#
    The authority for "which window belongs to this run" is the process itself:
    MainWindowHandle of the launched process (or of a direct child, e.g. the NSIS
    uninstaller which relaunches itself from %TEMP%). The enumeration fallback is
    only used while MainWindowHandle is still 0.
#>
function Get-OwnedMainWindow {
    param([int]$RootPid, [string]$TitlePart)
    $tree = Get-ProcessTreePids -RootPid $RootPid
    foreach ($pid2 in $tree) {
        $p = Get-Process -Id $pid2 -ErrorAction SilentlyContinue
        if (-not $p) { continue }
        $h = $p.MainWindowHandle
        if ($h -eq [IntPtr]::Zero -or -not [JediCap]::Alive($h)) { continue }
        if (-not [JediCap]::Visible($h)) { continue }
        if ([int][JediCap]::PidOf($h) -ne $pid2) { continue }
        if ($TitlePart -and ([JediCap]::Title($h) -notlike "*$TitlePart*")) { continue }
        $b = [JediCap]::Bounds($h)
        if ($b[2] -lt 200 -or $b[3] -lt 150) { continue }
        return $h
    }
    return [IntPtr]::Zero
}

$hwnd = [IntPtr]::Zero
$hwndSource = ''
$sw = [Diagnostics.Stopwatch]::StartNew()
while ($sw.ElapsedMilliseconds -lt $LaunchTimeoutMs) {
    $hwnd = Get-OwnedMainWindow -RootPid $proc.Id -TitlePart $TitleContains
    if ($hwnd -ne [IntPtr]::Zero) { $hwndSource = 'process MainWindowHandle'; break }
    $hwnd = Find-WizardWindow -RootPid $proc.Id -TitlePart $TitleContains
    if ($hwnd -ne [IntPtr]::Zero) { $hwndSource = 'process-tree window enumeration'; break }
    Start-Sleep -Milliseconds 200
}
if ($hwnd -eq [IntPtr]::Zero) {
    Write-Log "ERROR: no window owned by the launched process after $LaunchTimeoutMs ms (title filter: '$TitleContains')"
    try { $proc.Kill() } catch { }
    exit 1
}
$targetPid = [int][JediCap]::PidOf($hwnd)
$script:TargetPid = $targetPid
Write-Log "window   : 0x$($hwnd.ToString('X')) via $hwndSource"
Write-Log "owner    : pid $targetPid class=[$([JediCap]::ClassOf($hwnd))] title=[$([JediCap]::Title($hwnd))] rect=$([JediCap]::Rect($hwnd))"
Write-Log "policy   : exactly one window is driven; every action re-checks that it is still the same pid"

[JediCap]::Focus($hwnd) | Out-Null
Start-Sleep -Milliseconds 900
[JediCap]::Focus($hwnd) | Out-Null
Start-Sleep -Milliseconds 300

<#
    Guard used before every action and every screenshot: the target must still be
    alive AND still owned by the pid we attached to. This is what prevents the tool
    from ever driving a window that belongs to a concurrent run by another agent.
#>
function Test-TargetWindow {
    param([IntPtr]$Hwnd)
    if (-not [JediCap]::Alive($Hwnd)) { return 'gone' }
    if ([int][JediCap]::PidOf($Hwnd) -ne $script:TargetPid) { return 'foreign' }
    return 'ok'
}

# --------------------------------------------------------------------------- #
# page walking
# --------------------------------------------------------------------------- #
function Get-StableIdentity {
    param([IntPtr]$Hwnd, [int]$SettleMs)
    for ($i = 0; $i -lt 8; $i++) {
        $id = [JediCap]::StableIdentity($Hwnd, $SettleMs)
        if (-not [string]::IsNullOrWhiteSpace($id)) { return $id }
    }
    return ''
}

function Wait-IdentityChange {
    param([IntPtr]$Hwnd, [string]$CurrentId, [int]$TimeoutMs)
    $sw2 = [Diagnostics.Stopwatch]::StartNew()
    while ($sw2.ElapsedMilliseconds -lt $TimeoutMs) {
        if (-not [JediCap]::Alive($Hwnd)) { return 'gone' }
        $id = [JediCap]::StableIdentity($Hwnd, 120)
        if (-not [string]::IsNullOrWhiteSpace($id) -and $id -ne $CurrentId) { return 'changed' }
        Start-Sleep -Milliseconds 80
    }
    return 'timeout'
}

function Invoke-Advance {
    param([IntPtr]$Hwnd, [string]$CurrentId, [int]$WaitMs)

    # The wizard can advance on its own (the install page jumps to the finish page).
    # Re-read the page right before touching anything so we never click "Finish".
    if (-not [JediCap]::Alive($Hwnd)) { return 'gone' }
    $fresh = [JediCap]::StableIdentity($Hwnd, 120)
    if (-not [string]::IsNullOrWhiteSpace($fresh) -and $fresh -ne $CurrentId) { return 'changed' }
    if ([JediCap]::IsTerminalForward($Hwnd)) { return 'terminal' }
    if (-not [JediCap]::IsClickableForward($Hwnd)) {
        Write-Log "  forward button is '$([JediCap]::ForwardLabel($Hwnd))' - refusing to click it"
        return 'terminal'
    }

    $btn = [JediCap]::ForwardButton($Hwnd)
    if ($btn -ne [IntPtr]::Zero -and -not [JediCap]::Enabled($btn)) {
        Write-Log '  forward button disabled (installing) - waiting for it to become enabled'
        $sw3 = [Diagnostics.Stopwatch]::StartNew()
        while ($sw3.ElapsedMilliseconds -lt $WaitMs) {
            Start-Sleep -Milliseconds 150
            if (-not [JediCap]::Alive($Hwnd)) { return 'gone' }
            $fresh = [JediCap]::StableIdentity($Hwnd, 120)
            if (-not [string]::IsNullOrWhiteSpace($fresh) -and $fresh -ne $CurrentId) { return 'changed' }
            $btn = [JediCap]::ForwardButton($Hwnd)
            if ($btn -eq [IntPtr]::Zero -or [JediCap]::Enabled($btn)) { break }
        }
        if (-not [JediCap]::IsClickableForward($Hwnd)) { return 'terminal' }
    }

    # 1) window-targeted click on the wizard's forward button.
    #    BM_CLICK goes straight to the dialog procedure: no foreground/focus needed,
    #    and it is ignored while the button is disabled (i.e. during installation).
    $btn = [JediCap]::ForwardButton($Hwnd)
    if ($btn -ne [IntPtr]::Zero -and [JediCap]::Enabled($btn) -and [JediCap]::IsClickableForward($Hwnd)) {
        [JediCap]::Click($btn)
        if ((Wait-IdentityChange -Hwnd $Hwnd -CurrentId $CurrentId -TimeoutMs 2500) -ne 'timeout') { return 'changed' }
    }

    # 2) window-targeted Enter (goes through the app's message loop -> IsDialogMessage)
    if ([JediCap]::IsClickableForward($Hwnd)) {
        [JediCap]::PostEnter($Hwnd)
        if ((Wait-IdentityChange -Hwnd $Hwnd -CurrentId $CurrentId -TimeoutMs 1500) -ne 'timeout') { return 'changed' }
    }

    # 3) last resort: a real global keystroke, only when the wizard really is foreground
    [JediCap]::Focus($Hwnd) | Out-Null
    Start-Sleep -Milliseconds 120
    if ([JediCap]::IsForeground($Hwnd) -and [JediCap]::IsClickableForward($Hwnd)) {
        Write-Log '  falling back to a global Enter keystroke'
        [JediCap]::SendEnter()
        if ((Wait-IdentityChange -Hwnd $Hwnd -CurrentId $CurrentId -TimeoutMs 2500) -ne 'timeout') { return 'changed' }
    }
    return 'timeout'
}

$shots = @()
$lastId = $null
$shotIdx = 0
$stall = 0
$lastWasInstalling = $false

# --------------------------------------------------------------------------- #
# state mode (single-window installer: idle -> progress -> done, no MUI pages)
# --------------------------------------------------------------------------- #
function Get-FrameSignature {
    param([IntPtr]$Hwnd)
    $b = [JediCap]::Bounds($Hwnd)
    $w = $b[2]; $h = $b[3]
    if ($w -le 0 -or $h -le 0) { return '' }
    $hb = [JediGdi]::GrabBitmap($b[0], $b[1], $w, $h)
    $bmp = [System.Drawing.Image]::FromHbitmap($hb)
    try {
        $sb = New-Object System.Text.StringBuilder
        for ($gy = 0; $gy -lt 18; $gy++) {
            for ($gx = 0; $gx -lt 24; $gx++) {
                $x = [Math]::Min([int]($w * ($gx + 0.5) / 24), $w - 1)
                $y = [Math]::Min([int]($h * ($gy + 0.5) / 18), $h - 1)
                $c = $bmp.GetPixel($x, $y)
                [void]$sb.Append(('{0:x1}{1:x1}{2:x1}' -f [int]($c.R / 16), [int]($c.G / 16), [int]($c.B / 16)))
            }
        }
        return $sb.ToString()
    } finally {
        $bmp.Dispose()
        [JediGdi]::FreeBitmap($hb) | Out-Null
    }
}

function Send-EnterKey {
    param([IntPtr]$Hwnd)
    [JediCap]::Focus($Hwnd) | Out-Null
    Start-Sleep -Milliseconds 100
    [JediCap]::PostEnter($Hwnd)
    Start-Sleep -Milliseconds 200
    if ([JediCap]::IsForeground($Hwnd)) { [JediCap]::SendEnter() }
}

function Invoke-ClickAtRelative {
    param([IntPtr]$Hwnd, [int]$X, [int]$Y)
    $b = [JediCap]::Bounds($Hwnd)
    $sx = $b[0] + $X
    $sy = $b[1] + $Y
    Write-Log "  click window-relative ($X,$Y) -> screen ($sx,$sy)"
    [JediCap]::Focus($Hwnd) | Out-Null
    Start-Sleep -Milliseconds 120
    [JediCap]::ClickScreen($sx, $sy)
}

function Invoke-StateAction {
    param([IntPtr]$Hwnd, [string]$Action)
    if ([string]::IsNullOrWhiteSpace($Action) -or $Action -eq 'none') { return }
    if ($Action -match '^wait:(\d+)$') { Start-Sleep -Milliseconds ([int]$Matches[1]); return }
    if ($Action -match '^click:(-?\d+),(-?\d+)$') {
        Invoke-ClickAtRelative -Hwnd $Hwnd -X ([int]$Matches[1]) -Y ([int]$Matches[2])
        return
    }
    if ($Action -match '^enter-or-click:(-?\d+),(-?\d+)$') {
        $cx = [int]$Matches[1]; $cy = [int]$Matches[2]
        $before = Get-FrameSignature -Hwnd $Hwnd
        Send-EnterKey -Hwnd $Hwnd
        Start-Sleep -Milliseconds 450
        $after = Get-FrameSignature -Hwnd $Hwnd
        if ($after -eq $before) {
            Write-Log '  Enter had no visible effect -> clicking instead'
            Invoke-ClickAtRelative -Hwnd $Hwnd -X $cx -Y $cy
        }
        return
    }
    if ($Action -eq 'enter') { Send-EnterKey -Hwnd $Hwnd; return }
    if ($Action -match '^press:(.+)$') {
        # click a control by its caption - works for owner-drawn "static buttons"
        # even when their window rect is wrong (the redesign paints the text itself)
        $label = $Matches[1]
        $ctrl = [JediCap]::ChildByText($Hwnd, $label)
        if ($ctrl -ne [IntPtr]::Zero) {
            Write-Log "  press control [$label] 0x$($ctrl.ToString('X'))"
            [JediCap]::PressChild($ctrl)
            return
        }
        Write-Log "  control [$label] not found; visible captions: $(([JediCap]::VisibleChildTexts($Hwnd) -join ' / '))"
        return
    }
    Write-Log "  unknown action '$Action' - ignored"
}

if ($States) {
    $prevHash = $null
    $stateList = @($States.Split(',') | ForEach-Object { $_.Trim() } | Where-Object { $_ })
    $actionList = @()
    if ($Actions) {
        # `;` is required when a coordinate contains a comma, e.g. "click:960,682;wait:2000"
        $sep = if ($Actions.Contains(';')) { ';' } else { ',' }
        $actionList = @($Actions.Split($sep) | ForEach-Object { $_.Trim() })
    }
    Write-Log "mode     : state sequence [$($stateList -join ' -> ')]"
    Write-Log "actions  : $(if ($actionList.Count) { $actionList -join ' | ' } else { '(enter)' })"

    try {
        Start-Sleep -Milliseconds 800
        for ($i = 0; $i -lt $stateList.Count; $i++) {
            $state = $stateList[$i]
            if ($i -gt 0) {
                $act = if (($i - 1) -lt $actionList.Count -and $actionList[$i - 1]) { $actionList[$i - 1] } else { 'enter' }
                Write-Log "  transition -> $state via '$act'"
                $guardAction = Test-TargetWindow -Hwnd $hwnd
                if ($guardAction -ne 'ok') {
                    Write-Log "  target window is $guardAction - stopping (never attach to another run)"
                    break
                }
                Invoke-StateAction -Hwnd $hwnd -Action $act
            }
            Start-Sleep -Milliseconds $StateWaitMs
            $guardShot = Test-TargetWindow -Hwnd $hwnd
            if ($guardShot -ne 'ok') { Write-Log "  target window is $guardShot before capture - stopping"; break }
            $safe = ($state -replace '[^A-Za-z0-9\-_]', '')
            $file = Join-Path $OutDir ('{0}-{1}-{2}.png' -f $Prefix, $Lang, $safe)
            $err = Save-WindowShot -Hwnd $hwnd -File $file
            if ($err) { Write-Log "  WARN state $state : $err" }
            if ($prevHash) {
                $hash = (Get-FileHash -LiteralPath $file -Algorithm MD5).Hash
                if ($hash -eq $prevHash) { Write-Log "  WARN state [$state] is byte-identical to the previous screenshot - the transition did not happen" }
                $prevHash = $hash
            } else {
                $prevHash = (Get-FileHash -LiteralPath $file -Algorithm MD5).Hash
            }
            $shots += [pscustomobject]@{ Index = ($i + 1); Label = $state; File = $file; Identity = '' }
            Write-Log "  state [$state] -> $(Split-Path -Leaf $file)  ($([JediCap]::Rect($hwnd)))"
        }
    } finally {
        if (-not $KeepWindow) {
            Write-Log 'cleaning up processes'
            try { & taskkill.exe /PID $proc.Id /T /F 2>&1 | Out-Null } catch { }
            try { $proc.WaitForExit(3000) | Out-Null } catch { }
        }
    }
    if ($SafetyLog) { Write-Snapshot -Label "after-run $Prefix/$Lang" }
    $residual1 = @(Get-Process -Name 'nsis-output', 'Au_', 'Un_A', 'Un_B' -ErrorAction SilentlyContinue)
    Write-Log "residual installer processes: $(if ($residual1.Count -gt 0) { ($residual1 | ForEach-Object { "$($_.ProcessName):$($_.Id)" }) -join ',' } else { 'none' })"
    Write-Log "done: $($shots.Count) state(s)"
    $shots | ForEach-Object { Write-Log ("  {0}  {1}" -f $_.Label.PadRight(12), (Split-Path -Leaf $_.File)) }
    if ($shots.Count -eq 0) { exit 2 }
    return
}

try {
    while ($shotIdx -lt $Pages -and $stall -lt $MaxStall) {
        if (-not [JediCap]::Alive($hwnd)) {
            # the NSIS uninstaller re-creates its window from a %TEMP% copy; try to re-attach once
            $again = [IntPtr]::Zero
            for ($k = 0; $k -lt 10 -and $again -eq [IntPtr]::Zero; $k++) {
                Start-Sleep -Milliseconds 200
                $again = Find-WizardWindow -RootPid $proc.Id -TitlePart $TitleContains
            }
            if ($again -eq [IntPtr]::Zero) { Write-Log '  window closed'; break }
            Write-Log "  re-attached to window 0x$($again.ToString('X'))"
            $hwnd = $again
            [JediCap]::Focus($hwnd) | Out-Null
            Start-Sleep -Milliseconds 500
        }

        $id = Get-StableIdentity -Hwnd $hwnd -SettleMs $StepDelayMs
        if ([string]::IsNullOrWhiteSpace($id)) {
            Write-Log '  empty page identity - retrying'
            $stall++
            Start-Sleep -Milliseconds 200
            continue
        }

        if ($id -ne $lastId) {
            # While the installer is busy the forward button is disabled and the
            # "current step" line keeps repainting. That is NOT a new page: never
            # emit a second screenshot for a page that is still installing.
            $disabledNow = $false
            $fb = [JediCap]::ForwardButton($hwnd)
            if ($fb -ne [IntPtr]::Zero -and -not [JediCap]::Enabled($fb)) { $disabledNow = $true }
            if ($lastWasInstalling -and $disabledNow) {
                Write-Log '  still installing on the same page - waiting'
                Start-Sleep -Milliseconds 250
                continue
            }
            $lastWasInstalling = $disabledNow

            $n = $shotIdx + 1
            $label = if ($n -le $labels.Count -and $labels[$n - 1]) { $labels[$n - 1] } else { "page-$('{0:D2}' -f $n)" }
            $label = ($label -replace '[^A-Za-z0-9\-_]', '')
            $file = Join-Path $OutDir ('{0}-{1}-{2:D2}-{3}.png' -f $Prefix, $Lang, $n, $label)
            $err = Save-WindowShot -Hwnd $hwnd -File $file
            if ($err) { Write-Log "  WARN shot $n ($label): $err" }
            $shots += [pscustomobject]@{ Index = $n; Label = $label; File = $file; Identity = $id }
            Write-Log "  shot $('{0:D2}' -f $n) [$label] -> $(Split-Path -Leaf $file)"
            $shotIdx++
            $lastId = $id
        } else {
            Write-Log '  page unchanged since the last shot - trying to advance again'
        }

        if ($shotIdx -ge $Pages) { break }

        if ([JediCap]::IsTerminalForward($hwnd)) {
            Write-Log '  last page reached (forward button is Finish/Close) - not advancing'
            break
        }

        $res = Invoke-Advance -Hwnd $hwnd -CurrentId $id -WaitMs $WaitPageMs
        if ($res -eq 'gone') { Write-Log '  window closed after advancing'; break }
        if ($res -eq 'timeout') {
            $stall++
            Write-Log "  no page change (stall $stall/$MaxStall)"
        } else {
            $stall = 0
        }
    }
} finally {
    if (-not $KeepWindow) {
        Write-Log 'cleaning up processes'
        # kill the launched process tree, then any leftover NSIS uninstaller copies
        try { & taskkill.exe /PID $proc.Id /T /F 2>&1 | Out-Null } catch { }
        foreach ($p in @(Get-Process -Name 'Au_', 'Un_A', 'Un_B', 'uninstall' -ErrorAction SilentlyContinue)) {
            try {
                if ($p.StartTime -ge $launchedAt.AddSeconds(-5) -and $p.Path -like "$OutDir*") { $p.Kill() }
            } catch { }
        }
        try { $proc.WaitForExit(3000) | Out-Null } catch { }
    }
}

$residual = @(Get-Process -Name 'nsis-output', 'Au_', 'Un_A', 'Un_B' -ErrorAction SilentlyContinue)
Write-Log "residual installer processes: $(if ($residual.Count -gt 0) { ($residual | ForEach-Object { "$($_.ProcessName):$($_.Id)" }) -join ',' } else { 'none' })"

if ($SafetyLog) { Write-Snapshot -Label "after-run $Prefix/$Lang" }

Write-Log "done: $($shots.Count) page(s)"
$shots | ForEach-Object { Write-Log ("  {0}  {1}" -f $_.Label.PadRight(12), (Split-Path -Leaf $_.File)) }
if ($shots.Count -eq 0) { exit 2 }
