; Tauri NSIS installer hooks for DeepServer 2.0.
; 2.0 installs over DeepServer 1.x (same product name and install folder). Tauri's upgrade path
; doesn't run the old uninstaller for /S installs, so this removes what 1.x left behind:
; its Explorer menu entries (they pass arguments 2.0 doesn't understand) and its extra programs.

!macro DS_DELETE_V1_MENU ROOT VERB
  DeleteRegKey ${ROOT} "Software\Classes\*\shell\${VERB}"
  DeleteRegKey ${ROOT} "Software\Classes\Directory\shell\${VERB}"
  DeleteRegKey ${ROOT} "Software\Classes\*\shell\${VERB}SelectLeft"
  DeleteRegKey ${ROOT} "Software\Classes\Directory\shell\${VERB}SelectLeft"
  DeleteRegKey ${ROOT} "Software\Classes\Directory\shell\${VERB}CopyVerify"
  DeleteRegKey ${ROOT} "Software\Classes\Drive\shell\${VERB}CopyVerify"
!macroend

!macro DS_DELETE_V1_DISKUSAGE_MENU ROOT
  DeleteRegKey ${ROOT} "Software\Classes\Directory\shell\DeepServer Disk Usage"
  DeleteRegKey ${ROOT} "Software\Classes\Drive\shell\DeepServer Disk Usage"
!macroend

!macro NSIS_HOOK_PREINSTALL
  SetRegView 64
  !insertmacro DS_DELETE_V1_MENU HKLM "DeepServer"
  !insertmacro DS_DELETE_V1_MENU HKCU "DeepServer"
  !insertmacro DS_DELETE_V1_MENU HKCU "OpenDiff"
  !insertmacro DS_DELETE_V1_DISKUSAGE_MENU HKLM
  !insertmacro DS_DELETE_V1_DISKUSAGE_MENU HKCU
  Delete "$INSTDIR\deepserver-cli.exe"
  Delete "$INSTDIR\open-diff-cli.exe"
  Delete "$INSTDIR\deepserver-diskusage.exe"
!macroend
