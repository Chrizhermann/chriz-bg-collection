; Installer hooks for Chriz Easy BG.
;
; Before installing: Tauri's installer writes each file over the installed one,
; but in update mode (the in-app updater runs it with /UPDATE), silently (/S)
; or after "Do not uninstall" it never removes files an older version shipped.
; A recipe file left next to its successor stops the app: two artifacts then
; disagree about one download (seen in Chriz Easy IWD, 0.1.3 to 0.1.6,
; 2026-11-04). So the folders the app bundles are emptied before the new files
; are written and hold exactly this version's files afterwards. The list matches
; the folders in bundle.resources (tests/package_config.rs).

!macro NSIS_HOOK_PREINSTALL
  ; The app reads these folders; it must be closed while they are replaced.
  !insertmacro CheckIfAppIsRunning "${MAINBINARYNAME}.exe" "${PRODUCTNAME}"
  !insertmacro CebgClearBundledFolder "manifest\artifacts"
  !insertmacro CebgClearBundledFolder "manifest\mods"
  !insertmacro CebgClearBundledFolder "manifest\presets"
  !insertmacro CebgClearBundledFolder "manifest\game-builds"
  !insertmacro CebgClearBundledFolder "manifest\releases"
  !insertmacro CebgClearBundledFolder "recipes\curated-full-current"
  !insertmacro CebgClearBundledFolder "hotpatches\bg"
!macroend

!macro CebgClearBundledFolder FOLDER
  Push "$INSTDIR\${FOLDER}"
  Call CebgRemoveTree
!macroend

; Removes a folder and everything in it. A junction or symbolic link inside is
; removed as a link and never entered, unlike RMDir /r. A file that cannot be
; removed stays; the install goes on and writes this version's files.
Function CebgRemoveTree
  Exch $0 ; the folder
  Push $1 ; search handle
  Push $2 ; entry name
  Push $3 ; entry attributes
  Push $4
  ClearErrors
  FindFirst $1 $2 "$0\*"
  cebg_tree_entry:
    StrCmp $2 "" cebg_tree_close
    StrCmp $2 "." cebg_tree_next
    StrCmp $2 ".." cebg_tree_next
    System::Call 'kernel32::GetFileAttributesW(w "$0\$2") i .r3'
    IntCmp $3 -1 cebg_tree_next
    IntOp $4 $3 & 0x10 ; FILE_ATTRIBUTE_DIRECTORY
    IntCmp $4 0 cebg_tree_file
    IntOp $4 $3 & 0x400 ; FILE_ATTRIBUTE_REPARSE_POINT: a junction or link
    IntCmp $4 0 0 cebg_tree_link cebg_tree_link
    Push "$0\$2"
    Call CebgRemoveTree
    Goto cebg_tree_next
  cebg_tree_link:
    ; Removes the link itself; its target stays as it is.
    RMDir "$0\$2"
    Goto cebg_tree_next
  cebg_tree_file:
    ; A file, or a link to a file: Delete removes the link, not its target.
    SetFileAttributes "$0\$2" NORMAL
    Delete "$0\$2"
  cebg_tree_next:
    FindNext $1 $2
    Goto cebg_tree_entry
  cebg_tree_close:
  FindClose $1
  RMDir "$0"
  ; A missing folder, as on a first install, is not an error for the installer.
  ClearErrors
  Pop $4
  Pop $3
  Pop $2
  Pop $1
  Pop $0
FunctionEnd

; Uninstalling, see src/application_data.rs:
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
