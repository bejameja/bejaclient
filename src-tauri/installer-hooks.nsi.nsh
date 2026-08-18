; Cross-runtime migration hook for the BejaClient Electron -> Tauri switch.
;
; Older BejaClient releases (v1.x) were Electron-based and, deliberately, share the
; exact same per-user install directory and registry uninstall key as this Tauri build
; ("BejaClient" — see UNINSTKEY / PRODUCTNAME in the generated installer.nsi). If an old
; Electron install is detected here, it's uninstalled silently first, so this installer
; lands in a clean directory instead of overlaying mismatched Electron/Chromium files
; (app.asar, locales/, *.pak, ffmpeg.dll, etc.) that this installer's own file list has
; no knowledge of and would never clean up on its own.
;
; Version-gated on "starts with 1." rather than an exact file/asar check so it stays
; correct without changes as Electron point releases keep shipping (all 1.x) and never
; fires against a previous Tauri install (2.x+) during normal self-updates.

!macro NSIS_HOOK_PREINSTALL
  ReadRegStr $R7 SHCTX "${UNINSTKEY}" "UninstallString"
  ReadRegStr $R8 SHCTX "${UNINSTKEY}" "DisplayVersion"
  ${If} $R7 != ""
  ${AndIf} $R8 != ""
    StrCpy $R9 $R8 1
    ${If} $R9 == "1"
      DetailPrint "Removing previous BejaClient (Electron) installation..."
      ExecWait '$R7 /S'
      Sleep 1000
    ${EndIf}
  ${EndIf}
!macroend
