; =============================================================================
; Jedi Forge —— Windows 安装包界面模板（NSIS 3 / MUI2，单窗口无向导 v2）
; -----------------------------------------------------------------------------
; 基线：Tauri CLI 2.10.1 内置 NSIS 模板
;   https://raw.githubusercontent.com/tauri-apps/tauri/tauri-cli-v2.10.1/
;   crates/tauri-bundler/src/bundle/windows/nsis/installer.nsi
; 设计规格：docs/installer-ux-redesign-2026-11-04.md
; 渲染方式：Tauri bundler 用 Handlebars 渲染本文件（占位符＝双大括号 + 变量名，
;   循环/条件段＝#each / #if … /each / /if），渲染结果以 UTF-8 BOM 写盘，
;   因此本文件里可以直接写中文（makensis 依据 BOM 识别 UTF-8）。
;
; v2 单窗口设计（没有 Welcome/目录/进度/完成 多页向导）：
;   * 安装器只注册一个自定义页（Page custom JediMainPage）：进入后隐藏 MUI 的八个
;     chrome 控件（Next/Back/Cancel/Branding.*/Line.*/Header.*），把窗口改成
;     760×520（按 DPI 缩放）并居中，再把内页对话框拉伸铺满客户区，得到一块
;     #0E0E12 的深色画布；所有文字用 static + CreateFont 绘制（矢量字体，
;     任何 DPI 都清晰，不使用大位图背景）。
;   * 同一窗口内用 $JediState 切换 4 个状态：0=待安装 1=安装中 2=完成 3=失败。
;   * 点「立即安装」后在页面里直接执行 Function JediRunInstall（按步骤更新进度条、
;     状态文字并 UpdateWindow），完成后停在完成态；点「启动/完成」后 Quit。
;     全程不进入 instfiles 页。
;   * 被动（/P）、静默（/S）、更新（/UPDATE）：自定义页直接 Abort，完全沿用原来的
;     instfiles 页 + Section Install 行为（含 SetAutoClose、/R、/ARGS、/NS）。
;   * 卸载器同样是一页单窗口（UninstPage custom un.JediUninstallPage）。
;
; 维护说明：
;   * 不要写死产品名 / 版本 / 路径这类字面量，一律用占位符或已有 !define；
;     渲染后的产物在 src-tauri/target/release/nsis/<arch>/installer.nsi，可对照。
;   * 新增用户可见文案必须同时提供 ${LANG_SIMPCHINESE} 与 ${LANG_ENGLISH}。
;   * 升级 Tauri CLI 后请重新取回新版官方模板 diff 本文件，重点核对：
;       1) Section Install / Section Uninstall 里被抽成 Function 的业务步骤
;          （File 列表、注册表、快捷方式、WebView2、NSIS_HOOK_*）是否与新版一致；
;       2) 被动 / 静默 / 更新 / /NS / /R / /ARGS 行为；
;       3) Section WebView2 / Section Install / Section Uninstall 现在只是
;          「调用同一个函数」的薄壳，业务逻辑不要写回 section，否则 GUI 模式不执行。
;
; 与 Tauri 默认模板的差异（合并清单）：
;   1) 页面：向导组合改为 [License]* [MultiUser]* [StartMenu(仅配置文件夹时)] +
;      Page custom JediMainPage + MUI_PAGE_INSTFILES（后者只服务被动/静默/更新）；
;      卸载器为 UninstPage custom un.JediUninstallPage + MUI_UNPAGE_INSTFILES。
;   2) Section Install 主体抽成 Function JediRunInstall（含钩子与 WebView2 检查），
;      Section Install 只保留被动模式的 SetAutoClose；Section WebView2 抽成
;      Function JediEnsureWebView2。
;   3) Section Uninstall 主体抽成 Function un.JediRunUninstall；删除 Tauri 用
;      CreateWindowEx 伪造的「删除应用数据」复选框，改成本窗口里可点击的 static
;      开关（状态仍写入 $DeleteAppDataCheckboxState）。
;   4) 桌面快捷方式由 $JediDesktopShortcutState 控制（默认创建，/NS 置 0），不再
;      依赖完成页上的 SHOWREADME 伪复选框；GUI 里是窗口内的开关。
;   5) 向导侧栏图 / 页头图不再使用（单窗口没有向导页），MUI_ICON 仍用 installerIcon；
;      窗口左上角 logo 直接用 installerIcon（白色 Koala，透明背景）加载成图标。
;   6) 语言：SimpChinese 用 Microsoft YaHei UI，English 用 Segoe UI（LangString ^Font）。
; =============================================================================

Unicode true
ManifestDPIAware true
; Add in `dpiAwareness` `PerMonitorV2` to manifest for Windows 10 1607+ (note this should not affect lower versions since they should be able to ignore this and pick up `dpiAware` `true` set by `ManifestDPIAware true`)
; Currently undocumented on NSIS's website but is in the Docs folder of source tree, see
; https://github.com/kichik/nsis/blob/5fc0b87b819a9eec006df4967d08e522ddd651c9/Docs/src/attributes.but#L286-L300
; https://github.com/tauri-apps/tauri/pull/10106
ManifestDPIAwareness PerMonitorV2

!if "{{compression}}" == "none"
  SetCompress off
!else
  ; Set the compression algorithm. We default to LZMA.
  SetCompressor /SOLID "{{compression}}"
!endif

!include MUI2.nsh
!include FileFunc.nsh
!include x64.nsh
!include WordFunc.nsh
!include "utils.nsh"
!include "FileAssociation.nsh"
!include "Win\COM.nsh"
!include "Win\Propkey.nsh"
!include "StrFunc.nsh"
${StrCase}
${StrLoc}



!define WEBVIEW2APPGUID "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"

!define MANUFACTURER "{{manufacturer}}"
!define PRODUCTNAME "{{product_name}}"
!define VERSION "{{version}}"
!define VERSIONWITHBUILD "{{version_with_build}}"
!define HOMEPAGE "{{homepage}}"
!define INSTALLMODE "{{install_mode}}"
!define LICENSE "{{license}}"
!define INSTALLERICON "{{installer_icon}}"
!define SIDEBARIMAGE "{{sidebar_image}}"
!define HEADERIMAGE "{{header_image}}"
!define MAINBINARYNAME "{{main_binary_name}}"
!define MAINBINARYSRCPATH "{{main_binary_path}}"
!define BUNDLEID "{{bundle_id}}"
!define COPYRIGHT "{{copyright}}"
!define OUTFILE "{{out_file}}"
!define ARCH "{{arch}}"
!define ADDITIONALPLUGINSPATH "{{additional_plugins_path}}"
!define ALLOWDOWNGRADES "{{allow_downgrades}}"
!define DISPLAYLANGUAGESELECTOR "{{display_language_selector}}"
!define INSTALLWEBVIEW2MODE "{{install_webview2_mode}}"
!define WEBVIEW2INSTALLERARGS "{{webview2_installer_args}}"
!define WEBVIEW2BOOTSTRAPPERPATH "{{webview2_bootstrapper_path}}"
!define WEBVIEW2INSTALLERPATH "{{webview2_installer_path}}"
!define MINIMUMWEBVIEW2VERSION "{{minimum_webview2_version}}"
!define UNINSTKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCTNAME}"
!define MANUKEY "Software\${MANUFACTURER}"
!define MANUPRODUCTKEY "${MANUKEY}\${PRODUCTNAME}"
!define UNINSTALLERSIGNCOMMAND "{{uninstaller_sign_cmd}}"
!define ESTIMATEDSIZE "{{estimated_size}}"
!define STARTMENUFOLDER "{{start_menu_folder}}"

; -----------------------------------------------------------------------------
; Jedi：品牌 / 布局常量（尺寸是 96 DPI 像素，运行时乘 $JediDpi/96）
; -----------------------------------------------------------------------------
!define JEDI_UI_W       440
!define JEDI_UI_H       600
!define JEDI_MARGIN    52
!define JEDI_BG        "FFFFFF"
!define JEDI_BG_CARD   "FFFFFF"
!define JEDI_BG_TRACK  "ECECEE"
!define JEDI_ACCENT    "111111"
!define JEDI_DANGER    "D14343"
!define JEDI_TEXT      "111111"
!define JEDI_TEXT_2    "6C707E"
!define JEDI_TEXT_3    "6C707E"
!define JEDI_BTN_W 336
!define JEDI_BTN_H 46
!define JEDI_ICON_PX 140

; Win32 常量（WinMessages.nsh 里可能没有，缺就补）
!define /ifndef STM_SETIMAGE    0x0172
!define /ifndef IMAGE_ICON      1
!define /ifndef LR_LOADFROMFILE 0x0010
!define /ifndef SS_NOTIFY       0x0100
!define /ifndef SS_CENTER       0x0001
!define /ifndef SS_CENTERIMAGE  0x0200
; static 按钮完整样式：WS_CHILD(0x40000000)|WS_VISIBLE(0x10000000)|SS_CENTER|SS_CENTERIMAGE|SS_NOTIFY
; 注意：NSIS 的 StrCpy 不会计算 "256 | 1" 这类表达式，必须先写成编译期常量。
!define JEDI_STYLE_BUTTON 0x50000301
!define JEDI_STYLE_LINK   0x50000101
!define /ifndef GWL_STYLE       0xFFFFFFF0
!define /ifndef GWLP_WNDPROC    0xFFFFFFFC
!define /ifndef SWP_NOZORDER    0x0004
!define /ifndef SWP_NOMOVE      0x0002
!define /ifndef DWMWA_DARK_NEW  20
!define /ifndef DWMWA_DARK_OLD  19

Var PassiveMode
Var UpdateMode
Var NoShortcutMode
Var WixMode
Var OldMainBinaryName

; Jedi：快捷方式状态（1 = 创建）
Var JediDesktopShortcutState
Var JediStartMenuShortcutState

; Jedi：单窗口 UI
Var JediDpi
Var JediDialog
Var JediClientW
Var JediClientH
Var JediState
Var JediFailText
Var JediStepText
Var JediProgressFullW
Var JediProgressH
Var JediLogoPx
Var JediUiActive
Var JediCaption
Var JediTagline
Var JediLocText
Var JediStepMsg
Var JediVersionText
Var JediTitleShown
Var JediTitleTimer
Var JediBtnY
Var JediSecY
Var JediFailSub
Var JediWebView2Failed
Var JediWebView2Error

; 控件句柄
Var JediLogo
Var JediWordmark
Var JediVersion
Var JediHeadline
Var JediSub
Var JediCard
Var JediCardTitle
Var JediCardPath
Var JediChange
Var JediPrimary
Var JediPrimaryBtn
Var JediSecondaryBtn
Var JediSecondary
Var JediProgressTrack
Var JediProgressFill
Var JediPercent
Var JediFooter
Var JediOption

; 字体句柄
Var JediFontHeadline
Var JediFontSub
Var JediFontWordmark
Var JediFontBody
Var JediFontSmall
Var JediFontButton

Name "${PRODUCTNAME}"
; 窗口标题：NSIS 每次切页都会重新套用 Caption，所以在编译期用 LangString 设置；
; 只在 .onGUIInit 里 SetWindowText 会被后续页面覆盖。Name 指令保持 Tauri 默认
; （它决定默认安装目录与注册表键），不在这里改。
Caption "$(jediSetupCaption)"
BrandingText "$(jediBrandingText)"
OutFile "${OUTFILE}"

; We don't actually use this value as default install path,
; it's just for nsis to append the product name folder in the directory selector
; https://nsis.sourceforge.io/Reference/InstallDir
!define PLACEHOLDER_INSTALL_DIR "placeholder\${PRODUCTNAME}"
InstallDir "${PLACEHOLDER_INSTALL_DIR}"

VIProductVersion "${VERSIONWITHBUILD}"
VIAddVersionKey "ProductName" "${PRODUCTNAME}"
VIAddVersionKey "FileDescription" "${PRODUCTNAME}"
VIAddVersionKey "LegalCopyright" "${COPYRIGHT}"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "ProductVersion" "${VERSION}"

# additional plugins
!addplugindir "${ADDITIONALPLUGINSPATH}"

; Uninstaller signing command
!if "${UNINSTALLERSIGNCOMMAND}" != ""
  !uninstfinalize '${UNINSTALLERSIGNCOMMAND}'
!endif

; Handle install mode, `perUser`, `perMachine` or `both`
!if "${INSTALLMODE}" == "perMachine"
  RequestExecutionLevel admin
!endif

!if "${INSTALLMODE}" == "currentUser"
  RequestExecutionLevel user
!endif

!if "${INSTALLMODE}" == "both"
  !define MULTIUSER_MUI
  !define MULTIUSER_INSTALLMODE_INSTDIR "${PRODUCTNAME}"
  !define MULTIUSER_INSTALLMODE_COMMANDLINE
  !if "${ARCH}" == "x64"
    !define MULTIUSER_USE_PROGRAMFILES64
  !else if "${ARCH}" == "arm64"
    !define MULTIUSER_USE_PROGRAMFILES64
  !endif
  !define MULTIUSER_INSTALLMODE_DEFAULT_REGISTRY_KEY "${UNINSTKEY}"
  !define MULTIUSER_INSTALLMODE_DEFAULT_REGISTRY_VALUENAME "CurrentUser"
  !define MULTIUSER_INSTALLMODEPAGE_SHOWUSERNAME
  !define MULTIUSER_INSTALLMODE_FUNCTION RestorePreviousInstallLocation
  !define MULTIUSER_EXECUTIONLEVEL Highest
  !include MultiUser.nsh
!endif

; Installer icon（同时作为单窗口左上角 logo）
!if "${INSTALLERICON}" != ""
  !define MUI_ICON "${INSTALLERICON}"
  !define MUI_UNICON "${INSTALLERICON}"
!endif

; 单窗口设计不再使用向导侧栏图 / 页头图：保留 SIDEBARIMAGE / HEADERIMAGE 变量，
; 但不再交给 MUI（否则 MUI 会加载用不到的位图、还会切换对话框模板）。

; Define registry key to store installer language
!define MUI_LANGDLL_REGISTRY_ROOT "HKCU"
!define MUI_LANGDLL_REGISTRY_KEY "${MANUPRODUCTKEY}"
!define MUI_LANGDLL_REGISTRY_VALUENAME "Installer Language"

; -----------------------------------------------------------------------------
; 页面序列
; -----------------------------------------------------------------------------
; 1. License（仅当配置了 license）
!if "${LICENSE}" != ""
  !define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
  !insertmacro MUI_PAGE_LICENSE "${LICENSE}"
!endif

; 2. 安装范围选择（仅 install_mode == both）
!if "${INSTALLMODE}" == "both"
  !define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
  !insertmacro MULTIUSER_PAGE_INSTALLMODE
!endif

; 3. 维护页（检测到已安装时的重装/升级/降级选择），保持 Tauri 原逻辑
Var ReinstallPageCheck
Page custom PageReinstall PageLeaveReinstall
Function PageReinstall
  ; Uninstall previous WiX installation if exists.
  ;
  ; A WiX installer stores the installation info in registry
  ; using a UUID and so we have to loop through all keys under
  ; `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall`
  ; and check if `DisplayName` and `Publisher` keys match ${PRODUCTNAME} and ${MANUFACTURER}
  ;
  ; This has a potential issue that there maybe another installation that matches
  ; our ${PRODUCTNAME} and ${MANUFACTURER} but wasn't installed by our WiX installer,
  ; however, this should be fine since the user will have to confirm the uninstallation
  ; and they can chose to abort it if doesn't make sense.
  StrCpy $0 0
  wix_loop:
    EnumRegKey $1 HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall" $0
    StrCmp $1 "" wix_loop_done ; Exit loop if there is no more keys to loop on
    IntOp $0 $0 + 1
    ReadRegStr $R0 HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$1" "DisplayName"
    ReadRegStr $R1 HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$1" "Publisher"
    StrCmp "$R0$R1" "${PRODUCTNAME}${MANUFACTURER}" 0 wix_loop
    ReadRegStr $R0 HKLM "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$1" "UninstallString"
    ${StrCase} $R1 $R0 "L"
    ${StrLoc} $R0 $R1 "msiexec" ">"
    StrCmp $R0 0 0 wix_loop_done
    StrCpy $WixMode 1
    StrCpy $R6 "SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\$1"
    Goto compare_version
  wix_loop_done:

  ; Check if there is an existing installation, if not, abort the reinstall page
  ReadRegStr $R0 SHCTX "${UNINSTKEY}" ""
  ReadRegStr $R1 SHCTX "${UNINSTKEY}" "UninstallString"
  ${IfThen} "$R0$R1" == "" ${|} Abort ${|}

  ; Compare this installar version with the existing installation
  ; and modify the messages presented to the user accordingly
  compare_version:
  StrCpy $R4 "$(older)"
  ${If} $WixMode = 1
    ReadRegStr $R0 HKLM "$R6" "DisplayVersion"
  ${Else}
    ReadRegStr $R0 SHCTX "${UNINSTKEY}" "DisplayVersion"
  ${EndIf}
  ${IfThen} $R0 == "" ${|} StrCpy $R4 "$(unknown)" ${|}

  nsis_tauri_utils::SemverCompare "${VERSION}" $R0
  Pop $R0
  ; Reinstalling the same version
  ${If} $R0 = 0
    StrCpy $R1 "$(alreadyInstalledLong)"
    StrCpy $R2 "$(addOrReinstall)"
    StrCpy $R3 "$(uninstallApp)"
    !insertmacro MUI_HEADER_TEXT "$(alreadyInstalled)" "$(chooseMaintenanceOption)"
  ; Upgrading
  ${ElseIf} $R0 = 1
    StrCpy $R1 "$(olderOrUnknownVersionInstalled)"
    StrCpy $R2 "$(uninstallBeforeInstalling)"
    StrCpy $R3 "$(dontUninstall)"
    !insertmacro MUI_HEADER_TEXT "$(alreadyInstalled)" "$(choowHowToInstall)"
  ; Downgrading
  ${ElseIf} $R0 = -1
    StrCpy $R1 "$(newerVersionInstalled)"
    StrCpy $R2 "$(uninstallBeforeInstalling)"
    StrCpy $R3 "$(dontUninstall)"
    !if "${ALLOWDOWNGRADES}" == "true"
      StrCpy $R3 "$(dontUninstall)"
    !else
      StrCpy $R3 "$(dontUninstallDowngrade)"
    !endif
    !insertmacro MUI_HEADER_TEXT "$(alreadyInstalled)" "$(choowHowToInstall)"
  ${Else}
    Abort
  ${EndIf}

  ; Skip showing the page if passive
  ;
  ; Note that we don't call this earlier at the begining
  ; of this function because we need to populate some variables
  ; related to current installed version if detected and whether
  ; we are downgrading or not.
  ${If} $PassiveMode = 1
    Call PageLeaveReinstall
  ${Else}
    nsDialogs::Create 1018
    Pop $R4
    ${IfThen} $(^RTL) = 1 ${|} nsDialogs::SetRTL $(^RTL) ${|}

    ${NSD_CreateLabel} 0 0 100% 24u $R1
    Pop $R1

    ${NSD_CreateRadioButton} 30u 50u -30u 8u $R2
    Pop $R2
    ${NSD_OnClick} $R2 PageReinstallUpdateSelection

    ${NSD_CreateRadioButton} 30u 70u -30u 8u $R3
    Pop $R3
    ; Disable this radio button if downgrading and downgrades are disabled
    !if "${ALLOWDOWNGRADES}" == "false"
      ${IfThen} $R0 = -1 ${|} EnableWindow $R3 0 ${|}
    !endif
    ${NSD_OnClick} $R3 PageReinstallUpdateSelection

    ; Check the first radio button if this the first time
    ; we enter this page or if the second button wasn't
    ; selected the last time we were on this page
    ${If} $ReinstallPageCheck <> 2
      SendMessage $R2 ${BM_SETCHECK} ${BST_CHECKED} 0
    ${Else}
      SendMessage $R3 ${BM_SETCHECK} ${BST_CHECKED} 0
    ${EndIf}

    ${NSD_SetFocus} $R2
    nsDialogs::Show
  ${EndIf}
FunctionEnd
Function PageReinstallUpdateSelection
  ${NSD_GetState} $R2 $R1
  ${If} $R1 == ${BST_CHECKED}
    StrCpy $ReinstallPageCheck 1
  ${Else}
    StrCpy $ReinstallPageCheck 2
  ${EndIf}
FunctionEnd
Function PageLeaveReinstall
  ${NSD_GetState} $R2 $R1

  ; If migrating from Wix, always uninstall
  ${If} $WixMode = 1
    Goto reinst_uninstall
  ${EndIf}

  ; In update mode, always proceeds without uninstalling
  ${If} $UpdateMode = 1
    Goto reinst_done
  ${EndIf}

  ; $R0 holds whether same(0)/upgrading(1)/downgrading(-1) version
  ; $R1 holds the radio buttons state:
  ;   1 => first choice was selected
  ;   0 => second choice was selected
  ${If} $R0 = 0 ; Same version, proceed
    ${If} $R1 = 1              ; User chose to add/reinstall
      Goto reinst_done
    ${Else}                    ; User chose to uninstall
      Goto reinst_uninstall
    ${EndIf}
  ${ElseIf} $R0 = 1 ; Upgrading
    ${If} $R1 = 1              ; User chose to uninstall
      Goto reinst_uninstall
    ${Else}
      Goto reinst_done         ; User chose NOT to uninstall
    ${EndIf}
  ${ElseIf} $R0 = -1 ; Downgrading
    ${If} $R1 = 1              ; User chose to uninstall
      Goto reinst_uninstall
    ${Else}
      Goto reinst_done         ; User chose NOT to uninstall
    ${EndIf}
  ${EndIf}

  reinst_uninstall:
    HideWindow
    ClearErrors

    ${If} $WixMode = 1
      ReadRegStr $R1 HKLM "$R6" "UninstallString"
      ExecWait '$R1' $0
    ${Else}
      ReadRegStr $4 SHCTX "${MANUPRODUCTKEY}" ""
      ReadRegStr $R1 SHCTX "${UNINSTKEY}" "UninstallString"
      ${IfThen} $UpdateMode = 1 ${|} StrCpy $R1 "$R1 /UPDATE" ${|} ; append /UPDATE
      ${IfThen} $PassiveMode = 1 ${|} StrCpy $R1 "$R1 /P" ${|} ; append /P
      StrCpy $R1 "$R1 _?=$4" ; append uninstall directory
      ExecWait '$R1' $0
    ${EndIf}

    BringToFront

    ${IfThen} ${Errors} ${|} StrCpy $0 2 ${|} ; ExecWait failed, set fake exit code

    ${If} $0 <> 0
    ${OrIf} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.exe"
      ; User cancelled wix uninstaller? return to select un/reinstall page
      ${If} $WixMode = 1
      ${AndIf} $0 = 1602
        Abort
      ${EndIf}

      ; User cancelled NSIS uninstaller? return to select un/reinstall page
      ${If} $0 = 1
        Abort
      ${EndIf}

      ; Other erros? show generic error message and return to select un/reinstall page
      MessageBox MB_ICONEXCLAMATION "$(unableToUninstall)"
      Abort
    ${EndIf}
  reinst_done:
FunctionEnd

; 4. Start Menu 目录页（仅配置了 STARTMENUFOLDER 时出现；保留 MUI_STARTMENU_* 宏）
Var AppStartMenuFolder
!if "${STARTMENUFOLDER}" != ""
  !define MUI_PAGE_CUSTOMFUNCTION_PRE SkipIfPassive
  !define MUI_STARTMENUPAGE_DEFAULTFOLDER "${STARTMENUFOLDER}"
!else
  !define MUI_PAGE_CUSTOMFUNCTION_PRE Skip
!endif
!insertmacro MUI_PAGE_STARTMENU Application $AppStartMenuFolder

; 5. Jedi 单窗口主页面（GUI 的全部界面都在这里）
Page custom JediMainPage

; 6. instfiles 页：只服务被动 / 静默 / 更新模式（GUI 在自定义页里 Quit）
!insertmacro MUI_PAGE_INSTFILES

Function RunMainBinary
  nsis_tauri_utils::RunAsUser "$INSTDIR\${MAINBINARYNAME}.exe" ""
FunctionEnd

; -----------------------------------------------------------------------------
; 卸载器页面
; -----------------------------------------------------------------------------
Var DeleteAppDataCheckboxState
!define MUI_UNCONFIRMPAGE_TEXT_TOP "$(jediUninstallConfirmText)"
!define MUI_PAGE_HEADER_TEXT "$(jediUninstallTitle)"
!define MUI_PAGE_HEADER_SUBTEXT "$(jediUninstallSubtitle)"
!define MUI_PAGE_CUSTOMFUNCTION_PRE un.SkipIfPassive
; 单窗口卸载器：一页搞定「确认 + 卸载 + 完成」
UninstPage custom un.JediUninstallPage
; 被动 / 更新模式仍然走原来的卸载进度页
!insertmacro MUI_UNPAGE_INSTFILES

;Languages

{{#each languages}}
!insertmacro MUI_LANGUAGE "{{this}}"
{{/each}}
!insertmacro MUI_RESERVEFILE_LANGDLL
{{#each language_files}}
  !include "{{this}}"
{{/each}}

; -----------------------------------------------------------------------------
; Jedi：字体覆盖（MUI 默认 SimpChinese 是「宋体 9pt」）
; -----------------------------------------------------------------------------
LangString ^Font ${LANG_SIMPCHINESE} "Microsoft YaHei UI"
LangString ^FontSize ${LANG_SIMPCHINESE} "9"
LangString ^Font ${LANG_ENGLISH} "Segoe UI"
LangString ^FontSize ${LANG_ENGLISH} "9"

; -----------------------------------------------------------------------------
; Jedi：自有文案（SimpChinese + English 必须成对）
; -----------------------------------------------------------------------------
LangString jediSetupCaption ${LANG_SIMPCHINESE} "Jedi Forge 安装程序"
LangString jediSetupCaption ${LANG_ENGLISH} "Jedi Forge Setup"
LangString jediUninstallCaption ${LANG_SIMPCHINESE} "Jedi Forge 卸载程序"
LangString jediUninstallCaption ${LANG_ENGLISH} "Jedi Forge Uninstall"
LangString ^UninstallCaption ${LANG_SIMPCHINESE} "Jedi Forge 卸载程序"
LangString ^UninstallCaption ${LANG_ENGLISH} "Jedi Forge Uninstall"
LangString jediBrandingText ${LANG_SIMPCHINESE} "Jedi Forge · 开发者实用工具集"
LangString jediBrandingText ${LANG_ENGLISH} "Jedi Forge · Developer utilities"

LangString jediBrandName ${LANG_SIMPCHINESE} "Jedi Forge"
LangString jediBrandName ${LANG_ENGLISH} "Jedi Forge"
LangString jediVersionPrefix ${LANG_SIMPCHINESE} "版本"
LangString jediVersionPrefix ${LANG_ENGLISH} "Version"

; 待安装态
LangString jediReadyTitle ${LANG_SIMPCHINESE} "安装 Jedi Forge"
LangString jediReadyTitle ${LANG_ENGLISH} "Install Jedi Forge"
LangString jediReadySubtitle ${LANG_SIMPCHINESE} "本地优先的开发者工具箱：Hosts 管理、知识壁纸、播客与 AI 助手。"
LangString jediReadySubtitle ${LANG_ENGLISH} "A local-first toolbox for developers: hosts, wallpapers, podcasts and AI."
LangString jediInstallLocationLabel ${LANG_SIMPCHINESE} "安装位置"
LangString jediInstallLocationLabel ${LANG_ENGLISH} "Install location"
LangString jediChangeLocation ${LANG_SIMPCHINESE} "更改"
LangString jediChangeLocation ${LANG_ENGLISH} "Change"
LangString jediSelectFolderTitle ${LANG_SIMPCHINESE} "选择 Jedi Forge的安装位置"
LangString jediSelectFolderTitle ${LANG_ENGLISH} "Choose the install location for Jedi Forge"
LangString jediWindowTitle ${LANG_SIMPCHINESE} "Jedi Forge 安装程序"
LangString jediWindowTitle ${LANG_ENGLISH} "Jedi Forge Setup"

LangString jediInstallNow ${LANG_SIMPCHINESE} "立即安装"
LangString jediInstallNow ${LANG_ENGLISH} "Install now"
LangString jediOptionOn ${LANG_SIMPCHINESE} "☑  创建桌面快捷方式"
LangString jediOptionOn ${LANG_ENGLISH} "☑  Create a desktop shortcut"
LangString jediOptionOff ${LANG_SIMPCHINESE} "☐  创建桌面快捷方式"
LangString jediOptionOff ${LANG_ENGLISH} "☐  Create a desktop shortcut"
LangString jediNoShortcutHint ${LANG_SIMPCHINESE} "安装到当前用户目录，无需管理员权限。"
LangString jediNoShortcutHint ${LANG_ENGLISH} "Installs to your user folder — no administrator rights required."
LangString jediCancel ${LANG_SIMPCHINESE} "取消"
LangString jediCancel ${LANG_ENGLISH} "Cancel"

; 安装中
LangString jediInstallingTitle ${LANG_SIMPCHINESE} "正在安装…"
LangString jediInstallingTitle ${LANG_ENGLISH} "Installing…"
LangString jediStepCheck ${LANG_SIMPCHINESE} "正在检查运行环境（WebView2）…"
LangString jediStepCheck ${LANG_ENGLISH} "Checking the runtime (WebView2)…"
LangString jediStepCopyApp ${LANG_SIMPCHINESE} "正在复制程序文件…"
LangString jediStepCopyApp ${LANG_ENGLISH} "Copying program files…"
LangString jediStepCopyRes ${LANG_SIMPCHINESE} "正在复制资源文件…"
LangString jediStepCopyRes ${LANG_ENGLISH} "Copying resources…"
LangString jediStepRegister ${LANG_SIMPCHINESE} "正在写入注册表与卸载信息…"
LangString jediStepRegister ${LANG_ENGLISH} "Registering the app…"
LangString jediStepShortcut ${LANG_SIMPCHINESE} "正在创建快捷方式…"
LangString jediStepShortcut ${LANG_ENGLISH} "Creating shortcuts…"
LangString jediStepFinish ${LANG_SIMPCHINESE} "正在完成收尾工作…"
LangString jediStepFinish ${LANG_ENGLISH} "Finishing up…"

; 完成态
LangString jediDoneTitle ${LANG_SIMPCHINESE} "安装完成"
LangString jediDoneTitle ${LANG_ENGLISH} "Installation complete"
LangString jediDoneSubtitle ${LANG_SIMPCHINESE} "Jedi Forge已就绪，可以从开始菜单或桌面快捷方式启动。"
LangString jediDoneSubtitle ${LANG_ENGLISH} "Jedi Forge is ready — launch it from the Start Menu or your desktop shortcut."
LangString jediRunApp ${LANG_SIMPCHINESE} "启动 Jedi Forge"
LangString jediRunApp ${LANG_ENGLISH} "Launch Jedi Forge"
LangString jediClose ${LANG_SIMPCHINESE} "完成"
LangString jediClose ${LANG_ENGLISH} "Done"
LangString jediDoneHint ${LANG_SIMPCHINESE} "感谢使用 Jedi Forge！"
LangString jediDoneHint ${LANG_ENGLISH} "Thanks for using Jedi Forge!"

; 失败态
LangString jediFailedTitle ${LANG_SIMPCHINESE} "安装失败"
LangString jediFailedTitle ${LANG_ENGLISH} "Installation failed"
LangString jediFailedDefault ${LANG_SIMPCHINESE} "安装过程中出现问题，请重试或联系开发者。"
LangString jediFailedDefault ${LANG_ENGLISH} "Something went wrong during the installation. Please try again."

; 卸载器
LangString jediUninstallTitle ${LANG_SIMPCHINESE} "卸载 Jedi Forge"
LangString jediUninstallTitle ${LANG_ENGLISH} "Uninstall Jedi Forge"
LangString jediUninstallSubtitle ${LANG_SIMPCHINESE} "将从本机移除 Jedi Forge，以及安装时创建的快捷方式。"
LangString jediUninstallSubtitle ${LANG_ENGLISH} "This removes Jedi Forge and the shortcuts created during installation."
LangString jediUninstallConfirmText ${LANG_SIMPCHINESE} "卸载会移除程序文件，以及安装时创建的快捷方式。"
LangString jediUninstallConfirmText ${LANG_ENGLISH} "This removes the program files and the shortcuts created during installation."
LangString jediDeleteAppDataOff ${LANG_SIMPCHINESE} "☐  同时删除应用数据（设置、缓存与日志）"
LangString jediDeleteAppDataOff ${LANG_ENGLISH} "☐  Also delete app data (settings, cache, logs)"
LangString jediDeleteAppDataOn ${LANG_SIMPCHINESE} "☑  同时删除应用数据（设置、缓存与日志）"
LangString jediDeleteAppDataOn ${LANG_ENGLISH} "☑  Also delete app data (settings, cache, logs)"
LangString jediUninstallNow ${LANG_SIMPCHINESE} "卸载"
LangString jediUninstallNow ${LANG_ENGLISH} "Uninstall"
LangString jediUninstallingTitle ${LANG_SIMPCHINESE} "正在卸载…"
LangString jediUninstallingTitle ${LANG_ENGLISH} "Uninstalling…"
LangString jediUninstallStepFiles ${LANG_SIMPCHINESE} "正在删除程序文件…"
LangString jediUninstallStepFiles ${LANG_ENGLISH} "Removing program files…"
LangString jediUninstallStepShortcuts ${LANG_SIMPCHINESE} "正在删除快捷方式与注册表项…"
LangString jediUninstallStepShortcuts ${LANG_ENGLISH} "Removing shortcuts and registry entries…"
LangString jediUninstallStepData ${LANG_SIMPCHINESE} "正在删除应用数据…"
LangString jediUninstallStepData ${LANG_ENGLISH} "Removing app data…"
LangString jediUninstallStepFinish ${LANG_SIMPCHINESE} "正在完成收尾工作…"
LangString jediUninstallStepFinish ${LANG_ENGLISH} "Finishing up…"
LangString jediUninstallDoneTitle ${LANG_SIMPCHINESE} "卸载完成"
LangString jediUninstallDoneTitle ${LANG_ENGLISH} "Uninstall complete"
LangString jediUninstallDoneSubtitle ${LANG_SIMPCHINESE} "Jedi Forge已从本机移除。"
LangString jediUninstallDoneSubtitle ${LANG_ENGLISH} "Jedi Forge has been removed from this computer."

; =============================================================================
; Jedi：单窗口 UI 基础件（同一份实现同时装到安装器与卸载器，PFX 为空或 "un."）
; =============================================================================

; 把 $0（96 DPI 像素）按当前 DPI 缩放
!macro JediDefScalePx PFX
  Function ${PFX}JediScalePx
    ${If} $JediDpi > 96
      IntOp $0 $0 * $JediDpi
      IntOp $0 $0 / 96
    ${EndIf}
  FunctionEnd
!macroend
!insertmacro JediDefScalePx ""

; 摆放控件：$1 = 控件句柄，$4..$7 = x,y,w,h（96 DPI）
!macro JediDefPlace PFX
  Function ${PFX}JediPlace
    StrCpy $0 $4
    Call ${PFX}JediScalePx
    StrCpy $8 $0
    StrCpy $0 $5
    Call ${PFX}JediScalePx
    StrCpy $9 $0
    StrCpy $0 $6
    Call ${PFX}JediScalePx
    StrCpy $2 $0
    StrCpy $0 $7
    Call ${PFX}JediScalePx
    StrCpy $3 $0
    System::Call 'user32::SetWindowPos(p r1, p 0, i r8, i r9, i r2, i r3, i ${SWP_NOZORDER})'
  FunctionEnd
!macroend
!insertmacro JediDefPlace ""

; 按「设备像素」摆放：$1 = 句柄，$4..$7 = x,y,w,h（已经是像素，不再缩放）
; 注意：System::Call 的 p/i 参数要用 r0..r9 引用寄存器，不能直接写 $Var，
; 所以这里统一走 $4..$7。
!macro JediDefPlaceAbs PFX
  Function ${PFX}JediPlaceAbs
    System::Call 'user32::SetWindowPos(p r1, p 0, i r4, i r5, i r6, i r7, i ${SWP_NOZORDER})'
  FunctionEnd
!macroend
!insertmacro JediDefPlaceAbs ""
!insertmacro JediDefPlaceAbs "un."

; 把控件移到屏幕外（SW_HIDE 在部分环境下重绘异常，隐藏改用移出）
!macro JediDefPark PFX
  Function ${PFX}JediPark
    ; $1 = 句柄。挪到 y=-200 保存原 y 到 $JediParkY<idx> 由调用方管理
    System::Call 'user32::SetWindowPos(p r1, p 0, i 0, i -200, i 0, i 0, i 0x15)'
  FunctionEnd
!macroend
!insertmacro JediDefPark ""
!insertmacro JediDefPark "un."

; 只改尺寸（保持位置）：$1 = 句柄，$6/$7 = 宽/高（设备像素）
!macro JediDefSizeAbs PFX
  Function ${PFX}JediSizeAbs
    System::Call 'user32::SetWindowPos(p r1, p 0, i 0, i 0, i r6, i r7, i 6)'
  FunctionEnd
!macroend
!insertmacro JediDefSizeAbs ""

; 给控件 OR 上额外样式（$0 = 样式位，$1 = 控件句柄）
; 注意：静态控件改样式后必须 SetWindowPos(SWP_FRAMECHANGED) 才会重画；
; 实测 nsDialogs::CreateControl 自定义 style 的静态在本机不显示，
; 所以统一走「NSD_CreateLabel + 改 style」。
!macro JediDefAddStyle PFX
  Function ${PFX}JediAddStyle
    System::Call 'user32::GetWindowLongW(p r1, i ${GWL_STYLE}) i.r2'
    IntOp $2 $2 | $0
    System::Call 'user32::SetWindowLongW(p r1, i ${GWL_STYLE}, i r2) i.r3'
    ; 剥离 nsDialogs 默认的 WS_EX_TRANSPARENT(0x20)——否则鼠标点击穿透，按钮点不着
    System::Call 'user32::SetWindowPos(p r1, p 0, i 0, i 0, i 0, i 0, i 0x27)'
  FunctionEnd
!macroend
!insertmacro JediDefAddStyle ""

; 准备窗口：深色标题栏、尺寸、居中，并取客户区大小（$JediCaption 为标题文案）
!macro JediDefPrepareWindow PFX
  Function ${PFX}JediPrepareWindow
    ; DPI
    StrCpy $JediDpi 96
    ClearErrors
    System::Call 'user32::GetDpiForWindow(p $HWNDPARENT) i.r0'
    ${IfNot} ${Errors}
    ${AndIf} $0 > 95
      StrCpy $JediDpi $0
    ${EndIf}

    ; 深色标题栏（Win10 20H1+ 用 20，旧版回退 19；失败忽略）
    ClearErrors
    System::Call 'dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i ${DWMWA_DARK_NEW}, *i 0, i 4) i.r0'
    ${If} $0 != 0
      System::Call 'dwmapi::DwmSetWindowAttribute(p $HWNDPARENT, i ${DWMWA_DARK_OLD}, *i 0, i 4)'
    ${EndIf}
    ClearErrors

    ; 标题（$JediCaption 由调用者在进入本函数前设好）
    System::Call 'user32::SetWindowTextW(p $HWNDPARENT, w "$JediCaption")'
    ClearErrors

    ; 尺寸
    StrCpy $0 ${JEDI_UI_W}
    Call ${PFX}JediScalePx
    StrCpy $2 $0
    StrCpy $0 ${JEDI_UI_H}
    Call ${PFX}JediScalePx
    StrCpy $3 $0

    ; 居中到工作区
    System::Call 'user32::SystemParametersInfoW(i 48, i 0, @r1, i 0)'
    System::Call '*$1(i .r4, i .r5, i .r6, i .r7)'
    IntOp $6 $6 - $4
    IntOp $7 $7 - $5
    IntOp $4 $6 - $2
    IntOp $4 $4 / 2
    IntOp $5 $7 - $3
    IntOp $5 $5 / 2
    System::Call 'user32::SetWindowPos(p $HWNDPARENT, p 0, i r4, i r5, i r2, i r3, i ${SWP_NOZORDER})'

    ; 客户区尺寸
    System::Call 'user32::GetClientRect(p $HWNDPARENT, @r1)'
    System::Call '*$1(i .r4, i .r5, i .r6, i .r7)'
    StrCpy $JediClientW $6
    StrCpy $JediClientH $7
  FunctionEnd
!macroend
!insertmacro JediDefPrepareWindow ""

; 隐藏 MUI 向导 chrome（八个控件），让窗口变成一块干净画布
!macro JediDefHideChrome PFX
  Function ${PFX}JediHideChrome
    ${If} $mui.Button.Next != 0
      ShowWindow $mui.Button.Next ${SW_HIDE}
    ${EndIf}
    ${If} $mui.Button.Back != 0
      ShowWindow $mui.Button.Back ${SW_HIDE}
    ${EndIf}
    ${If} $mui.Button.Cancel != 0
      ShowWindow $mui.Button.Cancel ${SW_HIDE}
    ${EndIf}
    ${If} $mui.Branding.Text != 0
      ShowWindow $mui.Branding.Text ${SW_HIDE}
    ${EndIf}
    ${If} $mui.Branding.Background != 0
      ShowWindow $mui.Branding.Background ${SW_HIDE}
    ${EndIf}
    ${If} $mui.Line.Standard != 0
      ShowWindow $mui.Line.Standard ${SW_HIDE}
    ${EndIf}
    ${If} $mui.Line.FullWindow != 0
      ShowWindow $mui.Line.FullWindow ${SW_HIDE}
    ${EndIf}
    ${If} $mui.Header.Background != 0
      ShowWindow $mui.Header.Background ${SW_HIDE}
    ${EndIf}
    ${If} $mui.Header.Text != 0
      ShowWindow $mui.Header.Text ${SW_HIDE}
    ${EndIf}
    ${If} $mui.Header.SubText != 0
      ShowWindow $mui.Header.SubText ${SW_HIDE}
    ${EndIf}
    ${If} $mui.Header.Image != 0
      ShowWindow $mui.Header.Image ${SW_HIDE}
    ${EndIf}
  FunctionEnd
!macroend
!insertmacro JediDefHideChrome ""

; 内页铺满客户区 + 满幅深色底
!macro JediDefCanvas PFX
  Function ${PFX}JediCanvas
    System::Call 'user32::SetWindowPos(p $JediDialog, p 0, i 0, i 0, i $JediClientW, i $JediClientH, i ${SWP_NOZORDER})'
    ; 注意：空文本的 static 在 Windows 上不会绘制背景，这里用空格（NSIS 常见做法）
    ${NSD_CreateLabel} 0 0 1 1 " "
    Pop $R8
    SetCtlColors $R8 "${JEDI_TEXT}" "${JEDI_BG}"
    StrCpy $1 $R8
    StrCpy $4 0
    StrCpy $5 0
    StrCpy $6 $JediClientW
    StrCpy $7 $JediClientH
    Call ${PFX}JediPlaceAbs
  FunctionEnd
!macroend
!insertmacro JediDefCanvas ""

; 更新进度条：$0 = 百分比；$JediStepText 已被调用者写好
!macro JediDefProgress PFX
  Function ${PFX}JediProgress
    IntOp $6 $JediProgressFullW * $0
    IntOp $6 $6 / 100
    StrCpy $7 $JediProgressH
    StrCpy $1 $JediProgressFill
    Call ${PFX}JediSizeAbs
    ${If} $JediStepText != 0
      ${NSD_SetText} $JediStepText "$JediStepMsg"
    ${EndIf}
    ${If} $JediPercent != 0
      ${NSD_SetText} $JediPercent "$0%"
    ${EndIf}
    System::Call 'user32::UpdateWindow(p $JediDialog)'
    System::Call 'user32::UpdateWindow(p $HWNDPARENT)'
    Sleep 120
  FunctionEnd
!macroend
!insertmacro JediDefProgress ""

!macro JediDefPlaceButtons PFX
  Function ${PFX}JediPlaceButtons
    StrCpy $0 ${JEDI_MARGIN}
    Call ${PFX}JediScalePx
    StrCpy $R0 $0
    StrCpy $0 ${JEDI_BTN_W}
    Call ${PFX}JediScalePx
    StrCpy $R1 $0
    StrCpy $0 ${JEDI_BTN_H}
    Call ${PFX}JediScalePx
    StrCpy $R2 $0
    StrCpy $R3 $JediClientW
    IntOp $R3 $R3 - $R0
    IntOp $R3 $R3 - $R1
    StrCpy $R4 $JediClientH
    IntOp $R4 $R4 - 66
    IntOp $R4 $R4 - $R2
    StrCpy $JediBtnY $R4
    StrCpy $1 $JediPrimary
    StrCpy $4 $R3
    StrCpy $5 $R4
    StrCpy $6 $R1
    StrCpy $7 $R2
    Call ${PFX}JediPlaceAbs
    StrCpy $1 $JediSecondary
    StrCpy $4 0
    StrCpy $5 $R4
    IntOp $5 $5 + $R2
    IntOp $5 $5 + 10
    StrCpy $6 $JediClientW
    StrCpy $7 20
    Call ${PFX}JediPlaceAbs
  FunctionEnd
!macroend
!insertmacro JediDefPlaceButtons ""
!insertmacro JediDefPlaceButtons "un."

; 创建字体
!macro JediFont VAR SIZE WEIGHT
  CreateFont ${VAR} "$(^Font)" "${SIZE}" "${WEIGHT}"
!macroend

; 摆一枚控件（宏包装）
!macro JediPlaceAt CTRL X Y W H
  StrCpy $1 ${CTRL}
  StrCpy $4 ${X}
  StrCpy $5 ${Y}
  StrCpy $6 ${W}
  StrCpy $7 ${H}
  Call JediPlace
!macroend

  ; 普通文字 static（居中）
!macro JediCreateHot OUT TEXT X Y W H FONT COLOR
  ${NSD_CreateLabel} 0 0 1 1 "${TEXT}"
  Pop ${OUT}
  SetCtlColors ${OUT} "${COLOR}" "${JEDI_BG}"
  SendMessage ${OUT} ${WM_SETFONT} ${FONT} 0
  StrCpy $1 ${OUT}
  StrCpy $0 0x201
  IntOp $0 $0 | 256
  Call JediAddStyle
  System::Call 'user32::SetWindowLongW(p ${OUT}, i -20, i 0x00000004)'
  !insertmacro JediPlaceAt ${OUT} ${X} ${Y} ${W} ${H}
!macroend

!macro JediText OUT TEXT X Y W H FONT COLOR
  ${NSD_CreateLabel} 0 0 1 1 "${TEXT}"
  Pop ${OUT}
  SetCtlColors ${OUT} "${COLOR}" "${JEDI_BG}"
  SendMessage ${OUT} ${WM_SETFONT} ${FONT} 0
  StrCpy $1 ${OUT}
  StrCpy $0 ${SS_CENTER}
  IntOp $0 $0 | 256
  Call JediAddStyle
  !insertmacro JediPlaceAt ${OUT} ${X} ${Y} ${W} ${H}
!macroend


; 卡片内部的文字 static：背景必须与卡片同色，否则会出现「亮色方块」

; 卡片内部的可点击文字（如「更改」）
!macro JediClickOnCard OUT TEXT X Y W H FONT COLOR STYLE
  ${NSD_CreateLabel} 0 0 1 1 "${TEXT}"
  Pop ${OUT}
  SetCtlColors ${OUT} "${COLOR}" "${JEDI_BG_CARD}"
  SendMessage ${OUT} ${WM_SETFONT} ${FONT} 0
  StrCpy $1 ${OUT}
  StrCpy $0 ${STYLE}
  IntOp $0 $0 | 256
  Call JediAddStyle
  !insertmacro JediPlaceAt ${OUT} ${X} ${Y} ${W} ${H}
!macroend

; 可点击 static（按钮 / 链接 / 开关）
!macro JediClick OUT TEXT X Y W H FONT COLOR BG STYLE
  ${NSD_CreateLabel} 0 0 1 1 "${TEXT}"
  Pop ${OUT}
  SetCtlColors ${OUT} "${COLOR}" "${BG}"
  SendMessage ${OUT} ${WM_SETFONT} ${FONT} 0
  StrCpy $1 ${OUT}
  StrCpy $0 ${STYLE}
  IntOp $0 $0 | 256
  Call JediAddStyle
  System::Call 'user32::SetWindowLongW(p ${OUT}, i -20, i 0x00000004)'
  System::Call 'user32::InvalidateRect(p $HWNDPARENT, p 0, i 1)'
  !insertmacro JediPlaceAt ${OUT} ${X} ${Y} ${W} ${H}
!macroend

; =============================================================================
; 安装器：主页面与状态机
; =============================================================================

; 切到某个状态：0=待安装 1=安装中 2=完成 3=失败
; =============================================================================
; 安装器：主页面与状态机（白底极简 · 440×600 · 居中单列）
; =============================================================================
; 状态：0=待安装 1=安装中 2=完成 3=失败
Function JediApplyState
  ${If} $JediState = 0
    ; 超长路径截尾（保留末 20 字符），保证「更改」同行右端不重叠
    StrLen $0 $INSTDIR
    ${If} $0 > 20
      StrCpy $1 $INSTDIR "" -20
      StrCpy $1 "…$1"
    ${Else}
      StrCpy $1 $INSTDIR
    ${EndIf}
    ${NSD_SetText} $JediLocText "安装位置  $1"
    ShowWindow $JediHeadline ${SW_HIDE}
    ShowWindow $JediSub ${SW_HIDE}
    ShowWindow $JediPercent ${SW_HIDE}
    ShowWindow $JediStepText ${SW_HIDE}
    ShowWindow $JediProgressTrack ${SW_HIDE}
    ShowWindow $JediProgressFill ${SW_HIDE}
    ShowWindow $JediLocText ${SW_SHOW}
    ShowWindow $JediChange ${SW_SHOW}
    ShowWindow $JediOption ${SW_SHOW}
    Call JediPlaceButtons
    ShowWindow $JediPrimary ${SW_SHOW}
    ShowWindow $JediSecondary ${SW_SHOW}
    ${NSD_SetText} $JediPrimary "$(jediInstallNow)"
    ${NSD_SetText} $JediSecondary "$(jediCancel)"
${ElseIf} $JediState = 1
    ${NSD_SetText} $JediPercent "0%"
    ${NSD_SetText} $JediStepText "$(jediStepCheck)"
    ShowWindow $JediHeadline ${SW_HIDE}
    ShowWindow $JediSub ${SW_HIDE}
    ShowWindow $JediLocText ${SW_HIDE}
    ShowWindow $JediChange ${SW_HIDE}
    ShowWindow $JediOption ${SW_HIDE}
    ShowWindow $JediPercent ${SW_SHOW}
    ShowWindow $JediStepText ${SW_SHOW}
    ShowWindow $JediProgressTrack ${SW_SHOW}
    ShowWindow $JediProgressFill ${SW_SHOW}
  ${ElseIf} $JediState = 2
    ${NSD_SetText} $JediHeadline "$(jediDoneTitle)"
    ${NSD_SetText} $JediSub "$(jediDoneSubtitle)"
    ${NSD_SetText} $JediPrimary "$(jediRunApp)"
    ${NSD_SetText} $JediSecondary "$(jediClose)"
    ShowWindow $JediPercent ${SW_HIDE}
    ShowWindow $JediStepText ${SW_HIDE}
    ShowWindow $JediProgressTrack ${SW_HIDE}
    ShowWindow $JediProgressFill ${SW_HIDE}
    ShowWindow $JediLocText ${SW_HIDE}
    ShowWindow $JediChange ${SW_HIDE}
    ShowWindow $JediOption ${SW_HIDE}
    ShowWindow $JediHeadline ${SW_SHOW}
    ShowWindow $JediSub ${SW_SHOW}
    ShowWindow $JediPrimary ${SW_SHOW}
    ShowWindow $JediSecondary ${SW_SHOW}
    Call JediPlaceButtons
    System::Call 'user32::InvalidateRect(p $JediPrimary, p 0, i 1)'
    System::Call 'user32::InvalidateRect(p $JediSecondary, p 0, i 1)'
${Else}
    ${NSD_SetText} $JediHeadline "$(jediFailedTitle)"
    ${NSD_SetText} $JediSub "$JediFailText"
    ${NSD_SetText} $JediPrimary "$(jediClose)"
    ShowWindow $JediSecondary ${SW_HIDE}
    ShowWindow $JediOption ${SW_HIDE}
    ShowWindow $JediChange ${SW_HIDE}
    ShowWindow $JediLocText ${SW_HIDE}
    ShowWindow $JediPercent ${SW_HIDE}
    ShowWindow $JediStepText ${SW_HIDE}
    ShowWindow $JediProgressTrack ${SW_HIDE}
    ShowWindow $JediProgressFill ${SW_HIDE}
    ShowWindow $JediHeadline ${SW_HIDE}
    ShowWindow $JediHeadline ${SW_SHOW}
    ShowWindow $JediSub ${SW_HIDE}
    ShowWindow $JediSub ${SW_SHOW}
    SetCtlColors $JediPrimary "FFFFFF" "B3261E"
    ShowWindow $JediPrimary ${SW_HIDE}
    ShowWindow $JediPrimary ${SW_SHOW}
    System::Call 'user32::InvalidateRect(p $JediPrimary, p 0, i 1)'
${EndIf}
  System::Call 'user32::UpdateWindow(p $JediDialog)'
  System::Call 'user32::UpdateWindow(p $HWNDPARENT)'
FunctionEnd

; 主按钮：待安装 → 开始安装；完成 → 启动应用；失败 → 退出
Function JediHoverTick
  System::Call 'user32::GetCursorPos(@r1)'
  System::Call '*$1(i .r2, i .r3)'
  System::Call 'user32::GetWindowRect(p $HWNDPARENT, @r4)'
  System::Call '*$4(i .r5, i .r6, i .r7, i .r8)'
  IntOp $R9 $3 - $6
  ${If} $2 u>= $5
  ${AndIf} $2 u<= $7
  ${AndIf} $R9 u>= 0
  ${AndIf} $R9 u<= 40
      ${If} $JediTitleShown = 0
        StrCpy $JediTitleShown 1
        System::Call 'user32::SetWindowTextW(p $HWNDPARENT, w "Jedi Forge 安装程序")'
        ; 首次悬停：剥离按钮 EXSTYLE 穿透位（一次性）
        System::Call 'user32::SetWindowLongW(p $JediPrimary, i -20, i 0x00000004)'
        System::Call 'user32::SetWindowLongW(p $JediSecondary, i -20, i 0x00000004)'
        System::Call 'user32::SetWindowLongW(p $JediChange, i -20, i 0x00000004)'
        System::Call 'user32::SetWindowLongW(p $JediOption, i -20, i 0x00000004)'
      ${EndIf}
  ${Else}
      ${If} $JediTitleShown = 1
        StrCpy $JediTitleShown 0
        System::Call 'user32::SetWindowTextW(p $HWNDPARENT, w "")'
      ${EndIf}
  ${EndIf}
FunctionEnd

Function JediPrimaryAction
  ${If} $JediState = 1
    ; 安装进行中：按钮虽可见但不可用，忽略点击
    Return
  ${EndIf}
  ${If} $JediState = 0
    StrCpy $JediState 1
    Call JediApplyState
    Call JediRunInstall
    ${If} $JediState = 1
      StrCpy $JediState 2
    ${EndIf}
    Call JediApplyState
  ${ElseIf} $JediState = 2
    Call RunMainBinary
    Quit
  ${Else}
    Quit
  ${EndIf}
FunctionEnd

Function JediSecondaryAction
  Quit
FunctionEnd

; 把「更改」铅笔图标摆到位置文字末端（文字居中，需实测宽度）
Function JediChangeAction
  nsDialogs::SelectFolderDialog "$(jediChangeLocation)" "$INSTDIR"
  Pop $0
  ${If} $0 == "error"
    Return
  ${EndIf}
  StrCpy $INSTDIR $0
  ; 超长路径截尾（保留末 20 字符），保证「更改」同行右端不重叠
  StrLen $2 $INSTDIR
  ${If} $2 > 20
    StrCpy $3 $INSTDIR "" -20
    StrCpy $3 "…$3"
  ${Else}
    StrCpy $3 $INSTDIR
  ${EndIf}
  ${NSD_SetText} $JediLocText "安装位置  $3"
FunctionEnd

Function JediOptionAction
  ${If} $JediDesktopShortcutState = 1
    StrCpy $JediDesktopShortcutState 0
    ${NSD_SetText} $JediOption "☐  $(jediOptionOn)"
  ${Else}
    StrCpy $JediDesktopShortcutState 1
    ${NSD_SetText} $JediOption "☑  $(jediOptionOn)"
  ${EndIf}
FunctionEnd

; =============================================================================
; 安装器：主页面
; =============================================================================
Function JediMainPage
  ; 被动 / 静默 / 更新：跳过自定义页，走 instfiles + Section Install
  ${If} $PassiveMode = 1
  ${OrIf} $UpdateMode = 1
  ${OrIf} ${Silent}
    Abort
  ${EndIf}

  StrCpy $JediUiActive 1
  StrCpy $JediCaption "$(jediWindowTitle)"
  Call JediPrepareWindow

  nsDialogs::Create 1018
  Pop $JediDialog
  ${If} $JediDialog == error
    Abort
  ${EndIf}
  ${IfThen} $(^RTL) = 1 ${|} nsDialogs::SetRTL $(^RTL) ${|}
  Call JediHideChrome
  Call JediCanvas

  ; 字体
  !insertmacro JediFont $JediFontHeadline 16 700
  !insertmacro JediFont $JediFontSub 10 400
  !insertmacro JediFont $JediFontWordmark 22 600
  !insertmacro JediFont $JediFontBody 10 400
  !insertmacro JediFont $JediFontSmall 9 400
  !insertmacro JediFont $JediFontButton 11 700

  ; 品牌：白底 App 图标（居中 140px，来自 bundle.windows.nsis.headerImage）
  StrCpy $0 ${JEDI_ICON_PX}
  Call JediScalePx
  StrCpy $JediLogoPx $0
  InitPluginsDir
  !if "{{header_image}}" != ""
    File "/oname=$PLUGINSDIR\jedi-app-icon.ico" "{{header_image}}"
  !endif
  ${NSD_CreateLabel} 0 0 1 1 ""
  Pop $JediLogo
  StrCpy $1 $JediLogo
  StrCpy $0 0x3
  IntOp $0 $0 | 256
  Call JediAddStyle
  !if "{{header_image}}" != ""
    System::Call 'user32::LoadImageW(p 0, w "$PLUGINSDIR\jedi-app-icon.ico", i ${IMAGE_ICON}, i $JediLogoPx, i $JediLogoPx, i ${LR_LOADFROMFILE}) p.r0'
    ${If} $0 != 0
      SendMessage $JediLogo ${STM_SETIMAGE} ${IMAGE_ICON} $0
    ${EndIf}
  !endif
  SetCtlColors $JediLogo "${JEDI_TEXT}" "${JEDI_BG}"
  !insertmacro JediPlaceAt $JediLogo 150 24 140 140

  ; 字标 + 标语
  !insertmacro JediText $JediWordmark "Jedi" 0 180 440 36 $JediFontWordmark "${JEDI_TEXT}"
  !insertmacro JediText $JediTagline "Y O U R   L O C A L   A I   A G E N T" 0 224 440 16 $JediFontSmall "${JEDI_TAG}"

  ; 安装位置 + 更改
  !insertmacro JediText $JediLocText "" 0 292 440 20 $JediFontSmall "${JEDI_MUTED}"
  ; 「更改」铅笔图标（LoadImageW 加载 sidebarImage 注入的 ico，位置紧随文字末端）
  nsDialogs::CreateControl STATIC "${DEFAULT_STYLES}|${SS_ICON}|${SS_NOTIFY}" "" 0 0 1 1 ""
  Pop $JediChange
  StrCpy $1 $JediChange
  StrCpy $0 0x3
  IntOp $0 $0 | 256
  Call JediAddStyle
  InitPluginsDir
  !if "{{sidebar_image}}" != ""
    File "/oname=$PLUGINSDIR\jedi-change.ico" "{{sidebar_image}}"
    System::Call 'user32::LoadImageW(p 0, w "$PLUGINSDIR\jedi-change.ico", i ${IMAGE_ICON}, i 16, i 16, i ${LR_LOADFROMFILE}) p.r0'
    ${If} $0 != 0
      SendMessage $JediChange ${STM_SETIMAGE} ${IMAGE_ICON} $0
    ${EndIf}
  !endif
  SetCtlColors $JediChange "${JEDI_ACCENT}" "${JEDI_BG}"
  ${NSD_OnClick} $JediChange JediChangeAction
  !insertmacro JediPlaceAt $JediChange 325 294 16 16

  ; 快捷方式开关（可点击文字开关：原生 checkbox 在自绘窗口不渲染）
  ; 进度（安装中显示）
  StrCpy $0 320
  Call JediScalePx
  StrCpy $JediProgressFullW $0
  StrCpy $0 4
  Call JediScalePx
  StrCpy $JediProgressH $0
  ${NSD_CreateLabel} 0 0 1 1 "0%"
  Pop $JediPercent
  SetCtlColors $JediPercent "${JEDI_TEXT}" "${JEDI_BG}"
  SendMessage $JediPercent ${WM_SETFONT} $JediFontWordmark 0
  StrCpy $1 $JediPercent
  StrCpy $0 ${SS_CENTER}
  IntOp $0 $0 | 256
  Call JediAddStyle
  !insertmacro JediPlaceAt $JediPercent 0 284 440 40
  ${NSD_CreateLabel} 0 0 1 1 ""
  Pop $JediProgressTrack
  SetCtlColors $JediProgressTrack "" "${JEDI_BG_TRACK}"
  !insertmacro JediPlaceAt $JediProgressTrack 60 308 320 4
  ${NSD_CreateLabel} 0 0 1 1 ""
  Pop $JediProgressFill
  SetCtlColors $JediProgressFill "" "${JEDI_ACCENT}"
  !insertmacro JediPlaceAt $JediProgressFill 60 308 160 4
  !insertmacro JediText $JediStepText "" 0 334 440 22 $JediFontSub "${JEDI_MUTED}"

  ; 标题 / 副标题（完成与失败态显示）
  !insertmacro JediText $JediHeadline "" 0 300 440 34 $JediFontHeadline "${JEDI_TEXT}"
  !insertmacro JediText $JediSub "" 0 344 440 20 $JediFontSub "${JEDI_MUTED}"

  ; 底部：主按钮 + 取消（位置统一由 JediPlaceButtons 处理）
  Call JediPlaceButtons

  ; 版本号（取消按钮下方的小字）
  !insertmacro JediText $JediVersionText "$(jediVersionPrefix) ${VERSION}" 0 548 440 18 $JediFontSmall "${JEDI_TEXT_3}"

    ${NSD_CreateLabel} 0 0 1 1 "$(jediInstallNow)"
  Pop $JediPrimary
  SetCtlColors $JediPrimary "FFFFFF" "${JEDI_ACCENT}"
  SendMessage $JediPrimary ${WM_SETFONT} $JediFontButton 0
  StrCpy $1 $JediPrimary
  StrCpy $0 0x201
  IntOp $0 $0 | 256
  Call JediAddStyle
  ${NSD_OnClick} $JediPrimary JediPrimaryAction

    ${NSD_CreateLabel} 0 0 1 1 "$(jediCancel)"
  Pop $JediSecondary
  SetCtlColors $JediSecondary "${JEDI_TEXT_2}" "${JEDI_BG}"
  SendMessage $JediSecondary ${WM_SETFONT} $JediFontSmall 0
  StrCpy $1 $JediSecondary
  StrCpy $0 0x201
  IntOp $0 $0 | 256
  Call JediAddStyle
  ${NSD_OnClick} $JediSecondary JediSecondaryAction


  ; 初始状态
  StrCpy $JediState 0
  StrCpy $JediFailText "$(jediFailedDefault)"
  Call JediApplyState

  ; 剥离按钮控件的 WS_EX_TRANSPARENT(0x20)——nsDialogs 默认带它，
  ; 会导致真实鼠标点击穿透到下层（预览 PostMessage 测不出）
  ${NSD_CreateTimer} JediHoverTick 200

  nsDialogs::Show

  ${NSD_KillTimer} JediHoverTick

  Quit
FunctionEnd

; 进度上报：GUI 里更新进度条，被动/静默模式只写 DetailPrint
Function JediReportStep
  ${If} $JediUiActive = 1
  ${AndIf} $JediState = 1
    Call JediProgress
  ${Else}
    DetailPrint "$JediStepText"
  ${EndIf}
FunctionEnd

Function JediEnsureWebView2
  StrCpy $JediWebView2Failed 0
  StrCpy $JediWebView2Error ""

  ; Check if Webview2 is already installed and skip this section
  ${If} ${RunningX64}
    ReadRegStr $4 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\${WEBVIEW2APPGUID}" "pv"
  ${Else}
    ReadRegStr $4 HKLM "SOFTWARE\Microsoft\EdgeUpdate\Clients\${WEBVIEW2APPGUID}" "pv"
  ${EndIf}
  ${If} $4 == ""
    ReadRegStr $4 HKCU "SOFTWARE\Microsoft\EdgeUpdate\Clients\${WEBVIEW2APPGUID}" "pv"
  ${EndIf}

  ${If} $4 == ""
    ; Webview2 installation
    ;
    ; Skip if updating
    ${If} $UpdateMode <> 1
      !if "${INSTALLWEBVIEW2MODE}" == "downloadBootstrapper"
        Delete "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        DetailPrint "$(webview2Downloading)"
        NSISdl::download "https://go.microsoft.com/fwlink/p/?LinkId=2124703" "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        Pop $0
        ${If} $0 == "success"
          DetailPrint "$(webview2DownloadSuccess)"
        ${Else}
          DetailPrint "$(webview2DownloadError)"
          StrCpy $JediWebView2Failed 1
          StrCpy $JediWebView2Error "$(webview2AbortError)"
          Return
        ${EndIf}
        StrCpy $6 "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        Goto install_webview2
      !endif

      !if "${INSTALLWEBVIEW2MODE}" == "embedBootstrapper"
        Delete "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        File "/oname=$TEMP\MicrosoftEdgeWebview2Setup.exe" "${WEBVIEW2BOOTSTRAPPERPATH}"
        DetailPrint "$(installingWebview2)"
        StrCpy $6 "$TEMP\MicrosoftEdgeWebview2Setup.exe"
        Goto install_webview2
      !endif

      !if "${INSTALLWEBVIEW2MODE}" == "offlineInstaller"
        Delete "$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe"
        File "/oname=$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe" "${WEBVIEW2INSTALLERPATH}"
        DetailPrint "$(installingWebview2)"
        StrCpy $6 "$TEMP\MicrosoftEdgeWebView2RuntimeInstaller.exe"
        Goto install_webview2
      !endif

      Goto webview2_done

      install_webview2:
        DetailPrint "$(installingWebview2)"
        ; $6 holds the path to the webview2 installer
        ExecWait "$6 ${WEBVIEW2INSTALLERARGS} /install" $1
        ${If} $1 = 0
          DetailPrint "$(webview2InstallSuccess)"
        ${Else}
          DetailPrint "$(webview2InstallError)"
          StrCpy $JediWebView2Failed 1
          StrCpy $JediWebView2Error "$(webview2AbortError)"
          Return
        ${EndIf}
      webview2_done:
    ${EndIf}
  ${Else}
    !if "${MINIMUMWEBVIEW2VERSION}" != ""
      ${VersionCompare} "${MINIMUMWEBVIEW2VERSION}" "$4" $R0
      ${If} $R0 = 1
        update_webview:
          DetailPrint "$(installingWebview2)"
          ${If} ${RunningX64}
            ReadRegStr $R1 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate" "path"
          ${Else}
            ReadRegStr $R1 HKLM "SOFTWARE\Microsoft\EdgeUpdate" "path"
          ${EndIf}
          ${If} $R1 == ""
            ReadRegStr $R1 HKCU "SOFTWARE\Microsoft\EdgeUpdate" "path"
          ${EndIf}
          ${If} $R1 != ""
            ; Chromium updater docs: https://source.chromium.org/chromium/chromium/src/+/main:docs/updater/user_manual.md
            ; Modified from "HKEY_LOCAL_MACHINE\SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Microsoft EdgeWebView\ModifyPath"
            ExecWait `"$R1" /install appguid=${WEBVIEW2APPGUID}&needsadmin=true` $1
            ${If} $1 = 0
              DetailPrint "$(webview2InstallSuccess)"
            ${Else}
              MessageBox MB_ICONEXCLAMATION|MB_ABORTRETRYIGNORE "$(webview2InstallError)" IDIGNORE ignore IDRETRY update_webview
              StrCpy $JediWebView2Failed 1
              StrCpy $JediWebView2Error "$(webview2AbortError)"
              Return
              ignore:
            ${EndIf}
          ${EndIf}
      ${EndIf}
    !endif
  ${EndIf}
FunctionEnd

; 真正的安装流程（GUI 与被动/静默共用）
Function JediRunInstall
  SetOutPath $INSTDIR

  !ifmacrodef NSIS_HOOK_PREINSTALL
    !insertmacro NSIS_HOOK_PREINSTALL
  !endif

  !insertmacro CheckIfAppIsRunning "${MAINBINARYNAME}.exe" "${PRODUCTNAME}"

  ; ---- 1. 运行环境（WebView2）----
  StrCpy $JediStepMsg "$(jediStepCheck)"
  StrCpy $0 5
  Call JediReportStep
  Call JediEnsureWebView2
  ${If} $JediWebView2Failed = 1
    StrCpy $JediFailText "$JediWebView2Error"
    StrCpy $JediState 3
    Return
  ${EndIf}

  ; ---- 2. 复制主程序 ----
  StrCpy $JediStepMsg "$(jediStepCopyApp)"
  StrCpy $0 25
  Call JediReportStep
  File "${MAINBINARYSRCPATH}"

  ; ---- 3. 复制资源 / 外部程序 / 关联 / 深链 ----
  StrCpy $JediStepMsg "$(jediStepCopyRes)"
  StrCpy $0 45
  Call JediReportStep

  {{#each resources_dirs}}
    CreateDirectory "$INSTDIR\\{{this}}"
  {{/each}}
  {{#each resources}}
    File /a "/oname={{this.[1]}}" "{{no-escape @key}}"
  {{/each}}

  ; Copy external binaries
  {{#each binaries}}
    File /a "/oname={{this}}" "{{no-escape @key}}"
  {{/each}}

  ; Create file associations
  {{#each file_associations as |association| ~}}
    {{#each association.ext as |ext| ~}}
       !insertmacro APP_ASSOCIATE "{{ext}}" "{{or association.name ext}}" "{{association-description association.description ext}}" "$INSTDIR\${MAINBINARYNAME}.exe,0" "Open with ${PRODUCTNAME}" "$INSTDIR\${MAINBINARYNAME}.exe $\"%1$\""
    {{/each}}
  {{/each}}
  ; Register deep links
  {{#each deep_link_protocols as |protocol| ~}}
    WriteRegStr SHCTX "Software\Classes\\{{protocol}}" "URL Protocol" ""
    WriteRegStr SHCTX "Software\Classes\\{{protocol}}" "" "URL:${BUNDLEID} protocol"
    WriteRegStr SHCTX "Software\Classes\\{{protocol}}\DefaultIcon" "" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\",0"
    WriteRegStr SHCTX "Software\Classes\\{{protocol}}\shell\open\command" "" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\" $\"%1$\""
  {{/each}}
  ; Create uninstaller
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Save $INSTDIR in registry for future installations
  WriteRegStr SHCTX "${MANUPRODUCTKEY}" "" $INSTDIR

  !if "${INSTALLMODE}" == "both"
    ; Save install mode to be selected by default for the next installation such as updating
    ; or when uninstalling
    WriteRegStr SHCTX "${UNINSTKEY}" $MultiUser.InstallMode 1
  !endif

  ; Remove old main binary if it doesn't match new main binary name
  ReadRegStr $OldMainBinaryName SHCTX "${UNINSTKEY}" "MainBinaryName"
  ${If} $OldMainBinaryName != ""
  ${AndIf} $OldMainBinaryName != "${MAINBINARYNAME}.exe"
    Delete "$INSTDIR\$OldMainBinaryName"
  ${EndIf}

  ; Save current MAINBINARYNAME for future updates
  WriteRegStr SHCTX "${UNINSTKEY}" "MainBinaryName" "${MAINBINARYNAME}.exe"

  ; Registry information for add/remove programs
  WriteRegStr SHCTX "${UNINSTKEY}" "DisplayName" "${PRODUCTNAME}"
  WriteRegStr SHCTX "${UNINSTKEY}" "DisplayIcon" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\""
  WriteRegStr SHCTX "${UNINSTKEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr SHCTX "${UNINSTKEY}" "Publisher" "${MANUFACTURER}"
  WriteRegStr SHCTX "${UNINSTKEY}" "InstallLocation" "$\"$INSTDIR$\""
  WriteRegStr SHCTX "${UNINSTKEY}" "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
  WriteRegDWORD SHCTX "${UNINSTKEY}" "NoModify" "1"
  WriteRegDWORD SHCTX "${UNINSTKEY}" "NoRepair" "1"

  ${GetSize} "$INSTDIR" "/M=uninstall.exe /S=0K /G=0" $0 $1 $2
  IntOp $0 $0 + ${ESTIMATEDSIZE}
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD SHCTX "${UNINSTKEY}" "EstimatedSize" "$0"

  !if "${HOMEPAGE}" != ""
    WriteRegStr SHCTX "${UNINSTKEY}" "URLInfoAbout" "${HOMEPAGE}"
    WriteRegStr SHCTX "${UNINSTKEY}" "URLUpdateInfo" "${HOMEPAGE}"
    WriteRegStr SHCTX "${UNINSTKEY}" "HelpLink" "${HOMEPAGE}"
  !endif

  ; ---- 5. 快捷方式 ----
  StrCpy $JediStepMsg "$(jediStepShortcut)"
  StrCpy $0 88
  Call JediReportStep

  ; Create start menu shortcut
  !insertmacro MUI_STARTMENU_WRITE_BEGIN Application
    Call CreateOrUpdateStartMenuShortcut
  !insertmacro MUI_STARTMENU_WRITE_END

  ; Create desktop shortcut according to the toggle (silent/passive default: create)
  ${If} $JediDesktopShortcutState = 1
    Call CreateOrUpdateDesktopShortcut
  ${EndIf}

  !ifmacrodef NSIS_HOOK_POSTINSTALL
    !insertmacro NSIS_HOOK_POSTINSTALL
  !endif

  ; ---- 6. 收尾 ----
  StrCpy $JediStepMsg "$(jediStepFinish)"
  StrCpy $0 100
  Call JediReportStep
FunctionEnd

; =============================================================================
; 卸载器：单窗口页面
; =============================================================================

; 卸载器 UI 基础件
!insertmacro JediDefScalePx "un."
!insertmacro JediDefPlace "un."
!insertmacro JediDefSizeAbs "un."
!insertmacro JediDefAddStyle "un."
!insertmacro JediDefPrepareWindow "un."
!insertmacro JediDefHideChrome "un."
!insertmacro JediDefCanvas "un."
!insertmacro JediDefProgress "un."

; 摆控件（卸载器版）
!macro JediUnPlaceAt CTRL X Y W H
  StrCpy $1 ${CTRL}
  StrCpy $4 ${X}
  StrCpy $5 ${Y}
  StrCpy $6 ${W}
  StrCpy $7 ${H}
  Call un.JediPlace
!macroend

!macro JediUnText OUT TEXT X Y W H FONT COLOR
  ${NSD_CreateLabel} 0 0 1 1 "${TEXT}"
  Pop ${OUT}
  SetCtlColors ${OUT} "${COLOR}" "${JEDI_BG}"
  SendMessage ${OUT} ${WM_SETFONT} ${FONT} 0
  StrCpy $1 ${OUT}
  StrCpy $0 ${SS_CENTER}
  Call un.JediAddStyle
  !insertmacro JediUnPlaceAt ${OUT} ${X} ${Y} ${W} ${H}
!macroend

; 卡片内部文字：背景必须与卡片同色
!macro JediUnTextOnCard OUT TEXT X Y W H FONT COLOR
  ${NSD_CreateLabel} 0 0 1 1 "${TEXT}"
  Pop ${OUT}
  SetCtlColors ${OUT} "${COLOR}" "${JEDI_BG_CARD}"
  SendMessage ${OUT} ${WM_SETFONT} ${FONT} 0
  !insertmacro JediUnPlaceAt ${OUT} ${X} ${Y} ${W} ${H}
!macroend

!macro JediUnClick OUT TEXT X Y W H FONT COLOR BG STYLE
  nsDialogs::CreateControl STATIC "${DEFAULT_STYLES}|${SS_CENTER}|${SS_CENTERIMAGE}|${SS_NOTIFY}" "" ${X} ${Y} ${W} ${H} "${TEXT}"
  Pop ${OUT}
  SetCtlColors ${OUT} "${COLOR}" "${BG}"
  SendMessage ${OUT} ${WM_SETFONT} ${FONT} 0
  StrCpy $1 ${OUT}
  StrCpy $0 ${STYLE}
  System::Call 'user32::SetWindowLongW(p ${OUT}, i -20, i 0x00000004)'
  System::Call 'user32::InvalidateRect(p $HWNDPARENT, p 0, i 1)'
  Call un.JediAddStyle
  !insertmacro JediUnPlaceAt ${OUT} ${X} ${Y} ${W} ${H}
!macroend

; 卸载器状态：0=确认 1=卸载中 2=完成
; =============================================================================
; 卸载器：状态机与主页面（白底极简 · 竖版单列）
; =============================================================================
; 状态：0=确认 1=卸载中 2=完成
Function un.JediApplyState
  ${If} $JediState = 0
    ; 超长路径截尾（保留末 20 字符），保证「更改」同行右端不重叠
    StrLen $0 $INSTDIR
    ${If} $0 > 20
      StrCpy $1 $INSTDIR "" -20
      StrCpy $1 "…$1"
    ${Else}
      StrCpy $1 $INSTDIR
    ${EndIf}
    ${NSD_SetText} $JediLocText "安装位置  $1"
    ShowWindow $JediHeadline ${SW_SHOW}
    ShowWindow $JediSub ${SW_SHOW}
    ShowWindow $JediLocText ${SW_SHOW}
    ShowWindow $JediOption ${SW_SHOW}
    ShowWindow $JediPercent ${SW_HIDE}
    ShowWindow $JediStepText ${SW_HIDE}
    ShowWindow $JediProgressTrack ${SW_HIDE}
    ShowWindow $JediProgressFill ${SW_HIDE}
    ShowWindow $JediPrimary ${SW_SHOW}
    ShowWindow $JediSecondary ${SW_SHOW}
    SetCtlColors $JediPrimary "FFFFFF" "${JEDI_DANGER}"
    ${NSD_SetText} $JediPrimary "$(jediUninstallNow)"
    Call un.JediPlaceButtons
    ${NSD_SetText} $JediSecondary "$(jediCancel)"
  ${ElseIf} $JediState = 1
    ${NSD_SetText} $JediPercent "0%"
    ${NSD_SetText} $JediStepText "$(jediUninstallStepFiles)"
    ShowWindow $JediHeadline ${SW_HIDE}
    ShowWindow $JediSub ${SW_HIDE}
    ShowWindow $JediLocText ${SW_HIDE}
    ShowWindow $JediOption ${SW_HIDE}
    ShowWindow $JediPercent ${SW_SHOW}
    ShowWindow $JediStepText ${SW_SHOW}
    ShowWindow $JediProgressTrack ${SW_SHOW}
    ShowWindow $JediProgressFill ${SW_SHOW}
  ${ElseIf} $JediState = 2
    ${NSD_SetText} $JediHeadline "$(jediUninstallDoneTitle)"
    ${NSD_SetText} $JediSub "$(jediUninstallDoneSubtitle)"
    ${NSD_SetText} $JediPrimary "$(jediClose)"
    ShowWindow $JediHeadline ${SW_SHOW}
    ShowWindow $JediSub ${SW_SHOW}
    ShowWindow $JediLocText ${SW_HIDE}
    ShowWindow $JediOption ${SW_HIDE}
    ShowWindow $JediPercent ${SW_HIDE}
    ShowWindow $JediStepText ${SW_HIDE}
    ShowWindow $JediProgressTrack ${SW_HIDE}
    ShowWindow $JediProgressFill ${SW_HIDE}
    ShowWindow $JediPrimary ${SW_SHOW}
    ShowWindow $JediSecondary ${SW_HIDE}
    SetCtlColors $JediPrimary "111111" "FFFFFF"
  ${EndIf}
  System::Call 'user32::UpdateWindow(p $JediDialog)'
  System::Call 'user32::UpdateWindow(p $HWNDPARENT)'
FunctionEnd

Function un.JediPrimaryAction
  ${If} $JediState = 0
    StrCpy $JediState 1
    Call un.JediApplyState
    Call un.JediRunUninstall
    ${If} $JediState = 1
      StrCpy $JediState 2
    ${EndIf}
    Call un.JediApplyState
  ${ElseIf} $JediState = 2
    Quit
  ${Else}
    Quit
  ${EndIf}
FunctionEnd

Function un.JediSecondaryAction
  Quit
FunctionEnd

Function un.JediOptionAction
  ${If} $DeleteAppDataCheckboxState = 1
    StrCpy $DeleteAppDataCheckboxState 0
    ${NSD_SetText} $JediOption "$(jediDeleteAppDataOff)"
  ${Else}
    StrCpy $DeleteAppDataCheckboxState 1
    ${NSD_SetText} $JediOption "$(jediDeleteAppDataOn)"
  ${EndIf}
FunctionEnd

; =============================================================================
; 卸载器：主页面
; =============================================================================
Function un.JediUninstallPage
  ${If} $PassiveMode = 1
  ${OrIf} $UpdateMode = 1
  ${OrIf} ${Silent}
    Abort
  ${EndIf}

  StrCpy $JediUiActive 1
  StrCpy $JediCaption "$(jediWindowTitle)"
  Call un.JediPrepareWindow

  nsDialogs::Create 1018
  Pop $JediDialog
  ${If} $JediDialog == error
    Abort
  ${EndIf}
  ${IfThen} $(^RTL) = 1 ${|} nsDialogs::SetRTL $(^RTL) ${|}
  Call un.JediHideChrome
  Call un.JediCanvas

  ; 字体
  !insertmacro JediFont $JediFontHeadline 16 700
  !insertmacro JediFont $JediFontSub 10 400
  !insertmacro JediFont $JediFontWordmark 22 600
  !insertmacro JediFont $JediFontBody 10 400
  !insertmacro JediFont $JediFontSmall 9 400
  !insertmacro JediFont $JediFontButton 11 700

  ; 品牌：白底 App 图标
  StrCpy $0 ${JEDI_ICON_PX}
  Call un.JediScalePx
  StrCpy $JediLogoPx $0
  InitPluginsDir
  !if "{{header_image}}" != ""
    File "/oname=$PLUGINSDIR\jedi-app-icon.ico" "{{header_image}}"
  !endif
  ${NSD_CreateLabel} 0 0 1 1 ""
  Pop $JediLogo
  StrCpy $1 $JediLogo
  StrCpy $0 0x3
  Call un.JediAddStyle
  !if "{{header_image}}" != ""
    System::Call 'user32::LoadImageW(p 0, w "$PLUGINSDIR\jedi-app-icon.ico", i ${IMAGE_ICON}, i $JediLogoPx, i $JediLogoPx, i ${LR_LOADFROMFILE}) p.r0'
    ${If} $0 != 0
      SendMessage $JediLogo ${STM_SETIMAGE} ${IMAGE_ICON} $0
    ${EndIf}
  !endif
  SetCtlColors $JediLogo "${JEDI_TEXT}" "${JEDI_BG}"
  !insertmacro JediUnPlaceAt $JediLogo 150 24 140 140

  ; 字标 + 标语
  !insertmacro JediUnText $JediWordmark "Jedi" 0 180 440 36 $JediFontWordmark "${JEDI_TEXT}"
  !insertmacro JediUnText $JediTagline "Y O U R   L O C A L   A I   A G E N T" 0 224 440 16 $JediFontSmall "${JEDI_TAG}"

  ; 标题 / 说明 / 位置
  !insertmacro JediUnText $JediHeadline "$(jediUninstallTitle)" 0 264 440 34 $JediFontHeadline "${JEDI_TEXT}"
  !insertmacro JediUnText $JediSub "$(jediUninstallSubtitle)" 0 310 440 20 $JediFontSub "${JEDI_MUTED}"
  !insertmacro JediUnText $JediLocText "" 0 354 440 20 $JediFontSmall "${JEDI_MUTED}"

  ; 数据开关（可点击文字开关）
  !insertmacro JediUnClick $JediOption "$(jediDeleteAppDataOff)" 0 392 440 24 $JediFontBody "${JEDI_TEXT}" "${JEDI_BG}" "${SS_CENTER}"
  ${NSD_OnClick} $JediOption un.JediOptionAction
  !insertmacro JediUnPlaceAt $JediOption 105 392 230 24

  ; 进度
  StrCpy $0 320
  Call un.JediScalePx
  StrCpy $JediProgressFullW $0
  StrCpy $0 4
  Call un.JediScalePx
  StrCpy $JediProgressH $0
  ${NSD_CreateLabel} 0 0 1 1 "0%"
  Pop $JediPercent
  SetCtlColors $JediPercent "${JEDI_TEXT}" "${JEDI_BG}"
  SendMessage $JediPercent ${WM_SETFONT} $JediFontWordmark 0
  StrCpy $1 $JediPercent
  StrCpy $0 ${SS_CENTER}
  Call un.JediAddStyle
  !insertmacro JediUnPlaceAt $JediPercent 0 420 440 40
  ${NSD_CreateLabel} 0 0 1 1 ""
  Pop $JediProgressTrack
  SetCtlColors $JediProgressTrack "" "${JEDI_BG_TRACK}"
  !insertmacro JediUnPlaceAt $JediProgressTrack 60 464 320 4
  ${NSD_CreateLabel} 0 0 1 1 ""
  Pop $JediProgressFill
  SetCtlColors $JediProgressFill "" "${JEDI_DANGER}"
  !insertmacro JediUnPlaceAt $JediProgressFill 60 464 160 4
  !insertmacro JediUnText $JediStepText "" 0 488 440 22 $JediFontSub "${JEDI_MUTED}"

  ; 底部按钮（统一摆放）
  Call un.JediPlaceButtons

  ${NSD_CreateLabel} 0 0 1 1 "$(jediUninstallNow)"
  Pop $JediPrimary
  SetCtlColors $JediPrimary "FFFFFF" "${JEDI_DANGER}"
  SendMessage $JediPrimary ${WM_SETFONT} $JediFontButton 0
  StrCpy $1 $JediPrimary
  StrCpy $0 0x201
  System::Call 'user32::SetWindowLongW(p $JediPrimary, i -20, i 0x00000004)'
  System::Call 'user32::InvalidateRect(p $HWNDPARENT, p 0, i 1)'
  Call un.JediAddStyle
  ${NSD_OnClick} $JediPrimary un.JediPrimaryAction
  StrCpy $1 $JediPrimary
  StrCpy $4 $R3
  StrCpy $5 $R4
  StrCpy $6 $R1
  StrCpy $7 $R2
  Call un.JediPlaceAbs

    ${NSD_CreateLabel} 0 0 1 1 "$(jediCancel)"
  Pop $JediSecondary
  SetCtlColors $JediSecondary "${JEDI_TEXT_2}" "${JEDI_BG}"
  SendMessage $JediSecondary ${WM_SETFONT} $JediFontSmall 0
  StrCpy $1 $JediSecondary
  StrCpy $0 0x201
  System::Call 'user32::SetWindowLongW(p $JediSecondary, i -20, i 0x00000004)'
  System::Call 'user32::InvalidateRect(p $HWNDPARENT, p 0, i 1)'
  Call un.JediAddStyle
  ${NSD_OnClick} $JediSecondary un.JediSecondaryAction
  StrCpy $1 $JediSecondary
  StrCpy $4 0
  StrCpy $5 $R4
  IntOp $5 $5 + $R2
  IntOp $5 $5 + 10
  StrCpy $6 $JediClientW
  StrCpy $7 20
  Call un.JediPlaceAbs

  ; 初始状态
  StrCpy $JediState 0
  Call un.JediApplyState

  nsDialogs::Show

  Quit
FunctionEnd

; 卸载进度上报
Function un.JediReportStep
  ${If} $JediUiActive = 1
  ${AndIf} $JediState = 1
    Call un.JediProgress
  ${Else}
    DetailPrint "$JediStepText"
  ${EndIf}
FunctionEnd

Function un.JediRunUninstall
  !ifmacrodef NSIS_HOOK_PREUNINSTALL
    !insertmacro NSIS_HOOK_PREUNINSTALL
  !endif

  !insertmacro CheckIfAppIsRunning "${MAINBINARYNAME}.exe" "${PRODUCTNAME}"

  ; ---- 1. 删除程序文件 ----
  StrCpy $JediStepMsg "$(jediUninstallStepFiles)"
  StrCpy $0 20
  Call un.JediReportStep

  ; Delete the app directory and its content from disk
  ; Copy main executable
  Delete "$INSTDIR\${MAINBINARYNAME}.exe"

  ; Delete resources
  {{#each resources}}
    Delete "$INSTDIR\\{{this.[1]}}"
  {{/each}}
  ; Delete external binaries
  {{#each binaries}}
    Delete "$INSTDIR\\{{this}}"
  {{/each}}
  ; Delete app associations
  {{#each file_associations as |association| ~}}
    {{#each association.ext as |ext| ~}}
      !insertmacro APP_UNASSOCIATE "{{ext}}" "{{or association.name ext}}"
    {{/each}}
  {{/each}}
  ; Delete deep links
  {{#each deep_link_protocols as |protocol| ~}}
    ReadRegStr $R7 SHCTX "Software\Classes\\{{protocol}}\shell\open\command" ""
    ${If} $R7 == "$\"$INSTDIR\${MAINBINARYNAME}.exe$\" $\"%1$\""
      DeleteRegKey SHCTX "Software\Classes\\{{protocol}}"
    ${EndIf}
  {{/each}}

  ; Delete uninstaller
  Delete "$INSTDIR\uninstall.exe"

  
  {{#each resources_ancestors}}
  RMDir /REBOOTOK "$INSTDIR\\{{this}}"
  {{/each}}  RMDir "$INSTDIR"

  ; ---- 2. 快捷方式与注册表 ----
  StrCpy $JediStepMsg "$(jediUninstallStepShortcuts)"
  StrCpy $0 55
  Call un.JediReportStep

  ; Remove shortcuts if not updating
  ${If} $UpdateMode <> 1
    !insertmacro DeleteAppUserModelId

    ; Remove start menu shortcut
    !insertmacro MUI_STARTMENU_GETFOLDER Application $AppStartMenuFolder
    !insertmacro IsShortcutTarget "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    Pop $0
    ${If} $0 = 1
      !insertmacro UnpinShortcut "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
      Delete "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
      RMDir "$SMPROGRAMS\$AppStartMenuFolder"
    ${EndIf}
    !insertmacro IsShortcutTarget "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    Pop $0
    ${If} $0 = 1
      !insertmacro UnpinShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk"
      Delete "$SMPROGRAMS\${PRODUCTNAME}.lnk"
    ${EndIf}

    ; Remove desktop shortcuts
    !insertmacro IsShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    Pop $0
    ${If} $0 = 1
      !insertmacro UnpinShortcut "$DESKTOP\${PRODUCTNAME}.lnk"
      Delete "$DESKTOP\${PRODUCTNAME}.lnk"
    ${EndIf}
  ${EndIf}

  ; Remove registry information for add/remove programs
  !if "${INSTALLMODE}" == "both"
    DeleteRegKey SHCTX "${UNINSTKEY}"
  !else if "${INSTALLMODE}" == "perMachine"
    DeleteRegKey HKLM "${UNINSTKEY}"
  !else
    DeleteRegKey HKCU "${UNINSTKEY}"
  !endif

  ; Removes the Autostart entry for ${PRODUCTNAME} from the HKCU Run key if it exists.
  ; This ensures the program does not launch automatically after uninstallation if it exists.
  ; If it doesn't exist, it does nothing.
  ; We do this when not updating (to preserve the registry value on updates)
  ${If} $UpdateMode <> 1
    DeleteRegValue HKCU "Software\Microsoft\Windows\CurrentVersion\Run" "${PRODUCTNAME}"
  ${EndIf}

  ; ---- 3. 应用数据（勾选时才删）----
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    StrCpy $JediStepMsg "$(jediUninstallStepData)"
    StrCpy $0 80
    Call un.JediReportStep

    ; Clear the install location $INSTDIR from registry
    DeleteRegKey SHCTX "${MANUPRODUCTKEY}"
    DeleteRegKey /ifempty SHCTX "${MANUKEY}"

    ; Clear the install language from registry
    DeleteRegValue HKCU "${MANUPRODUCTKEY}" "Installer Language"
    DeleteRegKey /ifempty HKCU "${MANUPRODUCTKEY}"
    DeleteRegKey /ifempty HKCU "${MANUKEY}"

    SetShellVarContext current
    RmDir /r "$APPDATA\${BUNDLEID}"
    RmDir /r "$LOCALAPPDATA\${BUNDLEID}"
  ${EndIf}

  !ifmacrodef NSIS_HOOK_POSTUNINSTALL
    !insertmacro NSIS_HOOK_POSTUNINSTALL
  !endif

  ; ---- 4. 收尾 ----
  StrCpy $JediStepMsg "$(jediUninstallStepFinish)"
  StrCpy $0 100
  Call un.JediReportStep
FunctionEnd

; =============================================================================
; Section 薄壳（只服务被动 / 静默 / 更新模式）
; =============================================================================

Section EarlyChecks
  ; Abort silent installer if downgrades is disabled
  !if "${ALLOWDOWNGRADES}" == "false"
  ${If} ${Silent}
    ; If downgrading
    ${If} $R0 = -1
      System::Call 'kernel32::AttachConsole(i -1)i.r0'
      ${If} $0 <> 0
        System::Call 'kernel32::GetStdHandle(i -11)i.r0'
        System::call 'kernel32::SetConsoleTextAttribute(i r0, i 0x0004)' ; set red color
        FileWrite $0 "$(silentDowngrades)"
      ${EndIf}
      Abort
    ${EndIf}
  ${EndIf}
  !endif

SectionEnd

Section WebView2
  ; 逻辑在 JediEnsureWebView2 里（GUI 模式从单窗口页调用同一个函数）
  StrCpy $JediUiActive 0
  Call JediEnsureWebView2
  ${If} $JediWebView2Failed = 1
    Abort "$JediWebView2Error"
  ${EndIf}
SectionEnd

Section Install
  ; 逻辑在 JediRunInstall 里（GUI 模式从单窗口页调用同一个函数）
  StrCpy $JediUiActive 0
  Call JediRunInstall

  ; Auto close this page for passive mode
  ${If} $PassiveMode = 1
    SetAutoClose true
  ${EndIf}
SectionEnd

Function .onInstSuccess
  ; Check for `/R` flag only in silent and passive installers because
  ; GUI installer has a toggle for the user to (re)start the app
  ${If} $PassiveMode = 1
  ${OrIf} ${Silent}
    ${GetOptions} $CMDLINE "/R" $R0
    ${IfNot} ${Errors}
      ${GetOptions} $CMDLINE "/ARGS" $R0
      nsis_tauri_utils::RunAsUser "$INSTDIR\${MAINBINARYNAME}.exe" "$R0"
    ${EndIf}
  ${EndIf}
FunctionEnd

Function .onInit
  ${GetOptions} $CMDLINE "/P" $PassiveMode
  ${IfNot} ${Errors}
    StrCpy $PassiveMode 1
  ${EndIf}

  ${GetOptions} $CMDLINE "/NS" $NoShortcutMode
  ${IfNot} ${Errors}
    StrCpy $NoShortcutMode 1
  ${EndIf}

  ${GetOptions} $CMDLINE "/UPDATE" $UpdateMode
  ${IfNot} ${Errors}
    StrCpy $UpdateMode 1
  ${EndIf}

  ; Jedi: 快捷方式默认值。被动 / 静默 / 更新模式不会显示自定义页，
  ; 因此这里的默认值保持旧行为（创建桌面快捷方式）；/NS 仍然两个都不建。
  StrCpy $JediDesktopShortcutState 1
  StrCpy $JediStartMenuShortcutState 1
  ${If} $NoShortcutMode = 1
    StrCpy $JediDesktopShortcutState 0
    StrCpy $JediStartMenuShortcutState 0
  ${EndIf}
  StrCpy $JediUiActive 0
  StrCpy $JediState 0

  !if "${DISPLAYLANGUAGESELECTOR}" == "true"
    !insertmacro MUI_LANGDLL_DISPLAY
  !endif

  !insertmacro SetContext

  ${If} $INSTDIR == "${PLACEHOLDER_INSTALL_DIR}"
    ; Set default install location
    !if "${INSTALLMODE}" == "perMachine"
      ${If} ${RunningX64}
        !if "${ARCH}" == "x64"
          StrCpy $INSTDIR "$PROGRAMFILES64\${PRODUCTNAME}"
        !else if "${ARCH}" == "arm64"
          StrCpy $INSTDIR "$PROGRAMFILES64\${PRODUCTNAME}"
        !else
          StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCTNAME}"
        !endif
      ${Else}
        StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCTNAME}"
      ${EndIf}
    !else if "${INSTALLMODE}" == "currentUser"
      StrCpy $INSTDIR "$LOCALAPPDATA\${PRODUCTNAME}"
    !endif

    Call RestorePreviousInstallLocation
  ${EndIf}


  !if "${INSTALLMODE}" == "both"
    !insertmacro MULTIUSER_INIT
  !endif
FunctionEnd

Function un.onInit
  !insertmacro SetContext

  !if "${INSTALLMODE}" == "both"
    !insertmacro MULTIUSER_UNINIT
  !endif

  !insertmacro MUI_UNGETLANGUAGE

  ${GetOptions} $CMDLINE "/P" $PassiveMode
  ${IfNot} ${Errors}
    StrCpy $PassiveMode 1
  ${EndIf}

  ${GetOptions} $CMDLINE "/UPDATE" $UpdateMode
  ${IfNot} ${Errors}
    StrCpy $UpdateMode 1
  ${EndIf}

  StrCpy $JediUiActive 0
  StrCpy $JediState 0
  StrCpy $DeleteAppDataCheckboxState 0
FunctionEnd

Section Uninstall
  ; 逻辑在 un.JediRunUninstall 里（GUI 模式从单窗口页调用同一个函数）
  StrCpy $JediUiActive 0
  Call un.JediRunUninstall

  ; Auto close if passive mode or updating
  ${If} $PassiveMode = 1
  ${OrIf} $UpdateMode = 1
    SetAutoClose true
  ${EndIf}
SectionEnd

Function RestorePreviousInstallLocation
  ReadRegStr $4 SHCTX "${MANUPRODUCTKEY}" ""
  StrCmp $4 "" +2 0
    StrCpy $INSTDIR $4
FunctionEnd

Function Skip
  Abort
FunctionEnd

Function SkipIfPassive
  ${IfThen} $PassiveMode = 1  ${|} Abort ${|}
FunctionEnd
Function un.SkipIfPassive
  ${IfThen} $PassiveMode = 1  ${|} Abort ${|}
FunctionEnd

Function CreateOrUpdateStartMenuShortcut
  ; We used to use product name as MAINBINARYNAME
  ; migrate old shortcuts to target the new MAINBINARYNAME
  StrCpy $R0 0

  !insertmacro IsShortcutTarget "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\$OldMainBinaryName"
  Pop $0
  ${If} $0 = 1
    !insertmacro SetShortcutTarget "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    StrCpy $R0 1
  ${EndIf}

  !insertmacro IsShortcutTarget "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\$OldMainBinaryName"
  Pop $0
  ${If} $0 = 1
    !insertmacro SetShortcutTarget "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    StrCpy $R0 1
  ${EndIf}

  ${If} $R0 = 1
    Return
  ${EndIf}

  ; Skip creating shortcut if in update mode or no shortcut mode
  ; but always create if migrating from wix
  ${If} $WixMode = 0
    ${If} $UpdateMode = 1
    ${OrIf} $NoShortcutMode = 1
      Return
    ${EndIf}
  ${EndIf}

  !if "${STARTMENUFOLDER}" != ""
    CreateDirectory "$SMPROGRAMS\$AppStartMenuFolder"
    CreateShortcut "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\$AppStartMenuFolder\${PRODUCTNAME}.lnk"
  !else
    CreateShortcut "$SMPROGRAMS\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    !insertmacro SetLnkAppUserModelId "$SMPROGRAMS\${PRODUCTNAME}.lnk"
  !endif
FunctionEnd

Function CreateOrUpdateDesktopShortcut
  ; We used to use product name as MAINBINARYNAME
  ; migrate old shortcuts to target the new MAINBINARYNAME
  !insertmacro IsShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\$OldMainBinaryName"
  Pop $0
  ${If} $0 = 1
    !insertmacro SetShortcutTarget "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    Return
  ${EndIf}

  ; Skip creating shortcut if in update mode
  ; but always create if migrating from wix
  ;
  ; Jedi: 这里不再看 $NoShortcutMode —— 它现在只表示「不要开始菜单快捷方式」，
  ; 桌面快捷方式由 $JediDesktopShortcutState（/NS 时在 .onInit 里置 0）决定。
  ${If} $WixMode = 0
  ${AndIf} $UpdateMode = 1
    Return
  ${EndIf}

  CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
  !insertmacro SetLnkAppUserModelId "$DESKTOP\${PRODUCTNAME}.lnk"
FunctionEnd
