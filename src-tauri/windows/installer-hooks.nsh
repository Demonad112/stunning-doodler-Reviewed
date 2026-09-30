; Tauri NSIS installer hooks for DeepServer: Explorer context menu (all users).
; Key names match shell-core's in-app registration (verb key "DeepServer"), so an
; HKCU entry written by the app overlays the HKLM one instead of duplicating it.
; The desktop shortcut is Tauri's own (finish-page checkbox; always on /S installs).

; Defined first so the hooks below can use it.
!macro DS_DELETE_USER_MENU VERB
  DeleteRegKey HKCU "Software\Classes\*\shell\${VERB}"
  DeleteRegKey HKCU "Software\Classes\Directory\shell\${VERB}"
  DeleteRegKey HKCU "Software\Classes\*\shell\${VERB}SelectLeft"
  DeleteRegKey HKCU "Software\Classes\Directory\shell\${VERB}SelectLeft"
!macroend

; Disk-usage engine entries. The key name is the engine's own (wds::strWinDirStat), so the per-user entry
; its Options page can write overlays these instead of adding a second one.
!macro DS_DELETE_DISKUSAGE_MENU ROOT
  DeleteRegKey ${ROOT} "Software\Classes\Directory\shell\DeepServer Disk Usage"
  DeleteRegKey ${ROOT} "Software\Classes\Drive\shell\DeepServer Disk Usage"
!macroend

!macro NSIS_HOOK_PREINSTALL
  ; DeepServer replaces OpenDiff. Offer to remove an existing OpenDiff so Explorer
  ; doesn't show both menus. Silent (/S) installs remove it without asking.
  SetRegView 64
  ReadRegStr $R9 HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\OpenDiff" "UninstallString"
  ${If} $R9 != ""
    ${IfNot} ${Silent}
      MessageBox MB_YESNO|MB_ICONQUESTION "OpenDiff is installed on this PC. DeepServer replaces it.$\n$\nRemove OpenDiff now? (recommended)" IDNO ds_keep_opendiff
    ${EndIf}
    ExecWait '$R9 /S'
    ds_keep_opendiff:
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  SetRegView 64
  WriteRegStr HKLM "Software\Classes\*\shell\DeepServer" "MUIVerb" "Compare with DeepServer"
  WriteRegStr HKLM "Software\Classes\*\shell\DeepServer" "Icon" "$INSTDIR\${MAINBINARYNAME}.exe"
  WriteRegStr HKLM "Software\Classes\*\shell\DeepServer\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" --shell-compare "%1"'
  WriteRegStr HKLM "Software\Classes\Directory\shell\DeepServer" "MUIVerb" "Compare with DeepServer"
  WriteRegStr HKLM "Software\Classes\Directory\shell\DeepServer" "Icon" "$INSTDIR\${MAINBINARYNAME}.exe"
  WriteRegStr HKLM "Software\Classes\Directory\shell\DeepServer\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" --shell-compare "%1"'
  WriteRegStr HKLM "Software\Classes\*\shell\DeepServerSelectLeft" "MUIVerb" "Select Left File for Compare"
  WriteRegStr HKLM "Software\Classes\*\shell\DeepServerSelectLeft" "Icon" "$INSTDIR\${MAINBINARYNAME}.exe"
  WriteRegStr HKLM "Software\Classes\*\shell\DeepServerSelectLeft\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" --shell-compare --select-left "%1"'
  WriteRegStr HKLM "Software\Classes\Directory\shell\DeepServerSelectLeft" "MUIVerb" "Select Left Folder for Compare"
  WriteRegStr HKLM "Software\Classes\Directory\shell\DeepServerSelectLeft" "Icon" "$INSTDIR\${MAINBINARYNAME}.exe"
  WriteRegStr HKLM "Software\Classes\Directory\shell\DeepServerSelectLeft\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" --shell-compare --select-left "%1"'

  ; Only when the engine was bundled (release builds pass src-tauri/tauri.engine.conf.json).
  ${If} ${FileExists} "$INSTDIR\deepserver-diskusage.exe"
    WriteRegStr HKLM "Software\Classes\Directory\shell\DeepServer Disk Usage" "MUIVerb" "Analyze disk usage"
    WriteRegStr HKLM "Software\Classes\Directory\shell\DeepServer Disk Usage" "Icon" "$INSTDIR\deepserver-diskusage.exe"
    WriteRegStr HKLM "Software\Classes\Directory\shell\DeepServer Disk Usage\command" "" '"$INSTDIR\deepserver-diskusage.exe" "%1"'
    WriteRegStr HKLM "Software\Classes\Drive\shell\DeepServer Disk Usage" "MUIVerb" "Analyze disk usage"
    WriteRegStr HKLM "Software\Classes\Drive\shell\DeepServer Disk Usage" "Icon" "$INSTDIR\deepserver-diskusage.exe"
    WriteRegStr HKLM "Software\Classes\Drive\shell\DeepServer Disk Usage\command" "" '"$INSTDIR\deepserver-diskusage.exe" "%1"'
  ${EndIf}

  ; Stale per-user entries left by OpenDiff's in-app registration point at a removed exe.
  !insertmacro DS_DELETE_USER_MENU "OpenDiff"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  SetRegView 64
  DeleteRegKey HKLM "Software\Classes\*\shell\DeepServer"
  DeleteRegKey HKLM "Software\Classes\Directory\shell\DeepServer"
  DeleteRegKey HKLM "Software\Classes\*\shell\DeepServerSelectLeft"
  DeleteRegKey HKLM "Software\Classes\Directory\shell\DeepServerSelectLeft"
  ; Entries the app itself wrote for the uninstalling user (Settings > Shell integration).
  !insertmacro DS_DELETE_USER_MENU "DeepServer"
  !insertmacro DS_DELETE_DISKUSAGE_MENU HKLM
  !insertmacro DS_DELETE_DISKUSAGE_MENU HKCU
!macroend
