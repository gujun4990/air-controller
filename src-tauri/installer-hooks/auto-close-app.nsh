!macro TryExitExistingApp
  IfFileExists "$INSTDIR\${MAINBINARYNAME}.exe" 0 +3
    ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --exit'
    Sleep 1500

  nsExec::ExecToLog 'taskkill /F /T /IM "${MAINBINARYNAME}.exe"'
  Sleep 500
!macroend

!macro NSIS_HOOK_PREINSTALL
  !insertmacro TryExitExistingApp
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  !insertmacro TryExitExistingApp
!macroend
