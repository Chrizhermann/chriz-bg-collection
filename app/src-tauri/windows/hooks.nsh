; Uninstaller hooks for Chriz Easy BG. See src/application_data.rs.
;
; With "Delete the application data" checked, Tauri's uninstaller deletes
; $APPDATA\${BUNDLEID} and $LOCALAPPDATA\${BUNDLEID} with RMDir /r. That leaves
; paths longer than 260 characters behind, such as mods unpacked in the download
; cache, and follows directory junctions out of these folders. The app removes
; both folders first, before the uninstaller deletes it.

!macro NSIS_HOOK_PREUNINSTALL
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    ; The app must be closed while its data is removed.
    !insertmacro CheckIfAppIsRunning "${MAINBINARYNAME}.exe" "${PRODUCTNAME}"
    ClearErrors
    ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --remove-application-data "${BUNDLEID}"' $0
    ${If} ${Errors}
    ${OrIf} $0 <> 0
      ; RMDir /r must not continue on what is left, because it follows junctions.
      StrCpy $DeleteAppDataCheckboxState 0
      SetShellVarContext current
      MessageBox MB_OK|MB_ICONEXCLAMATION "${PRODUCTNAME} could not remove all of its application data. If you no longer need it, delete these folders yourself:$\r$\n$\r$\n$LOCALAPPDATA\${BUNDLEID}$\r$\n$APPDATA\${BUNDLEID}" /SD IDOK
    ${EndIf}
  ${EndIf}
!macroend
