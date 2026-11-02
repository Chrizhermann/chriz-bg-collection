; Uninstaller hooks for Chriz Easy BG. See src/application_data.rs.
;
; With "Delete the application data" checked, Tauri's uninstaller deletes
; $APPDATA\${BUNDLEID} and $LOCALAPPDATA\${BUNDLEID} with RMDir /r. That leaves
; paths longer than 260 characters behind, such as mods unpacked in the download
; cache, and follows directory junctions out of these folders. The app removes
; both folders instead, before the uninstaller deletes it, and RMDir /r never
; runs. The template's other cleanup for the checkbox runs after uninstalling.

Var CebgDeleteAppData
Var CebgAppDataLeft

!macro NSIS_HOOK_PREUNINSTALL
  StrCpy $CebgDeleteAppData 0
  StrCpy $CebgAppDataLeft 0
  ${If} $DeleteAppDataCheckboxState = 1
  ${AndIf} $UpdateMode <> 1
    StrCpy $CebgDeleteAppData 1
    ; RMDir /r must never run, whatever the app removes: it follows junctions,
    ; also ones another program creates after the app is done.
    StrCpy $DeleteAppDataCheckboxState 0
    ; The app must be closed while its data is removed.
    !insertmacro CheckIfAppIsRunning "${MAINBINARYNAME}.exe" "${PRODUCTNAME}"
    ClearErrors
    ExecWait '"$INSTDIR\${MAINBINARYNAME}.exe" --remove-application-data "${BUNDLEID}"' $0
    ${If} ${Errors}
    ${OrIf} $0 <> 0
      StrCpy $CebgAppDataLeft 1
    ${EndIf}
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $CebgDeleteAppData = 1
    ; The template's registry cleanup for the checkbox, which no longer runs there.
    DeleteRegKey SHCTX "${MANUPRODUCTKEY}"
    DeleteRegKey /ifempty SHCTX "${MANUKEY}"
    DeleteRegValue HKCU "${MANUPRODUCTKEY}" "Installer Language"
    DeleteRegKey /ifempty HKCU "${MANUPRODUCTKEY}"
    DeleteRegKey /ifempty HKCU "${MANUKEY}"
    ${If} $CebgAppDataLeft = 1
      SetShellVarContext current
      MessageBox MB_OK|MB_ICONEXCLAMATION "${PRODUCTNAME} could not remove all of its application data. If you no longer need it, delete these folders yourself:$\r$\n$\r$\n$LOCALAPPDATA\${BUNDLEID}$\r$\n$APPDATA\${BUNDLEID}" /SD IDOK
    ${EndIf}
  ${EndIf}
!macroend
