; Force the installer to request Administrator privileges
RequestExecutionLevel admin

!macro NSIS_HOOK_POSTINSTALL
  ; Register whatsapp://
  WriteRegStr SHCTX "Software\Classes\whatsapp" "" "URL:WhatsApped Protocol"
  WriteRegStr SHCTX "Software\Classes\whatsapp" "URL Protocol" ""
  WriteRegStr SHCTX "Software\Classes\whatsapp\shell" "" ""
  WriteRegStr SHCTX "Software\Classes\whatsapp\shell\open" "" ""
  WriteRegStr SHCTX "Software\Classes\whatsapp\shell\open\command" "" '"$INSTDIR\whatsapped.exe" "%1"'

  ; Register wapped://
  WriteRegStr SHCTX "Software\Classes\wapped" "" "URL:WhatsApped Protocol"
  WriteRegStr SHCTX "Software\Classes\wapped" "URL Protocol" ""
  WriteRegStr SHCTX "Software\Classes\wapped\shell" "" ""
  WriteRegStr SHCTX "Software\Classes\wapped\shell\open" "" ""
  WriteRegStr SHCTX "Software\Classes\wapped\shell\open\command" "" '"$INSTDIR\whatsapped.exe" "%1"'
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  ; Remove registry keys on uninstall
  DeleteRegKey SHCTX "Software\Classes\whatsapp"
  DeleteRegKey SHCTX "Software\Classes\wapped"
!macroend