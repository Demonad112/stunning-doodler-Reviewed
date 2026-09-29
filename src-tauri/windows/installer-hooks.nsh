; Tauri NSIS installer hooks: Explorer context menu (all users) and desktop shortcut.
; Key names/commands mirror scripts/windows/register-shell-extension.ps1 and shell-core.

!macro NSIS_HOOK_PREINSTALL
!macroend

!macro NSIS_HOOK_POSTINSTALL
  SetRegView 64
  WriteRegStr HKLM "Software\Classes\*\shell\OpenDiff" "MUIVerb" "Compare with Open Diff"
  WriteRegStr HKLM "Software\Classes\*\shell\OpenDiff" "Icon" "$INSTDIR\open-diff-app.exe"
  WriteRegStr HKLM "Software\Classes\*\shell\OpenDiff\command" "" '"$INSTDIR\open-diff-app.exe" --shell-compare "%1"'
  WriteRegStr HKLM "Software\Classes\Directory\shell\OpenDiff" "MUIVerb" "Compare with Open Diff"
  WriteRegStr HKLM "Software\Classes\Directory\shell\OpenDiff" "Icon" "$INSTDIR\open-diff-app.exe"
  WriteRegStr HKLM "Software\Classes\Directory\shell\OpenDiff\command" "" '"$INSTDIR\open-diff-app.exe" --shell-compare "%1"'
  WriteRegStr HKLM "Software\Classes\*\shell\OpenDiffSelectLeft" "MUIVerb" "Select Left File for Compare"
  WriteRegStr HKLM "Software\Classes\*\shell\OpenDiffSelectLeft" "Icon" "$INSTDIR\open-diff-app.exe"
  WriteRegStr HKLM "Software\Classes\*\shell\OpenDiffSelectLeft\command" "" '"$INSTDIR\open-diff-app.exe" --shell-compare --select-left "%1"'
  WriteRegStr HKLM "Software\Classes\Directory\shell\OpenDiffSelectLeft" "MUIVerb" "Select Left Folder for Compare"
  WriteRegStr HKLM "Software\Classes\Directory\shell\OpenDiffSelectLeft" "Icon" "$INSTDIR\open-diff-app.exe"
  WriteRegStr HKLM "Software\Classes\Directory\shell\OpenDiffSelectLeft\command" "" '"$INSTDIR\open-diff-app.exe" --shell-compare --select-left "%1"'

  SetShellVarContext all
  CreateShortcut "$DESKTOP\${PRODUCTNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
!macroend

!macro NSIS_HOOK_PREUNINSTALL
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  SetRegView 64
  DeleteRegKey HKLM "Software\Classes\*\shell\OpenDiff"
  DeleteRegKey HKLM "Software\Classes\Directory\shell\OpenDiff"
  DeleteRegKey HKLM "Software\Classes\*\shell\OpenDiffSelectLeft"
  DeleteRegKey HKLM "Software\Classes\Directory\shell\OpenDiffSelectLeft"
  SetShellVarContext all
  Delete "$DESKTOP\${PRODUCTNAME}.lnk"
!macroend
