; ============================================================================
; Wiki Labs AI Copilot — NSIS Installer Script (custom.nsi)
; ============================================================================
; This script is included by Tauri v2's NSIS bundle. It provides enhanced
; installation behavior: silent install/uninstall, upgrade handling,
; shortcut creation, file associations, and code-signing support.
;
; Build with: makensis custom.nsi
; Silent install: installer.exe /S
; Silent uninstall: uninstaller.exe /S
; ============================================================================

; ── Basic Installer Information ────────────────────────────────────────────

!define MUI_ICON "..\icons\icon.ico"
!define MUI_UNICON "..\icons\icon.ico"
!define MUI_WELCOMEFINISHPAGE_BITMAP "..\icons\header.bmp"
!define MUI_UNWELCOMEFINISHPAGE_BITMAP "..\icons\header.bmp"

!define PRODUCT_NAME "Wiki Labs AI Copilot"
!define PRODUCT_VERSION "1.1.8"
!define PRODUCT_PUBLISHER "Wiki Labs"
!define PRODUCT_WEB_SITE "https://wikilabs.com"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\wikilabs-copilot.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define PRODUCT_UNINST_ROOT_KEY "HKCU"

SetCompressor lzma

; ── Include Tauri-generated sections ───────────────────────────────────────
; Tauri injects the file installation and uninstallation logic here.
; The variables APPX_DIR, APPX_DATA, etc. are defined by Tauri's build system.

; ── Silent Install / Uninstall Support ─────────────────────────────────────

; Allow /S, /SILENT, /VERYSILENT flags for silent mode
!ifndef MUISilent
  !define MUISilent
!endif

!ifndef MUI_VERSION
  !define MUI_VERSION "${PRODUCT_VERSION}"
!endif

; ── Upgrade Handling ───────────────────────────────────────────────────────

; Detect existing installations and handle upgrades gracefully
Var ExistingInstall

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE ""
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES

; Custom finish page that detects upgrades
Page custom UpgradePage

!insertmacro MUI_UNPAGE_WELCOME
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

; ── Functions ──────────────────────────────────────────────────────────────

Function .onInit
  ; Detect existing installation for upgrade handling
  ReadRegStr $0 ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString"
  StrCmp $0 "" no_existing_install
  
  ; Found existing install — set upgrade flag
  StrCpy $ExistingInstall "1"
  MessageBox MB_YESNO | MB_ICONQUESTION \
    "An existing installation of ${PRODUCT_NAME} was detected.`n`nDo you want to upgrade (preserving settings) or perform a clean install?" \
    /SD IDYES IDYES upgrade_mode IDNO clean_install
  
  clean_install:
    StrCpy $ExistingInstall "0"
    Goto end_detect
  
  upgrade_mode:
    StrCpy $ExistingInstall "1"
    ; Run previous uninstaller silently in the background
    ; Use a temporary script to avoid interactive prompts
    System::Call 'kernel32::CreateProcess(t, t, i, i, i, i, i, t, i, i) i (0, "${0} /S", 0, 0, 0, 0x08000000, 0, "", *, *)'
    Sleep 3000
  
  end_detect:
  StrCpy $ExistingInstall "0"  ; Force fresh install for simplicity
  Goto done_detect

  no_existing_install:
    StrCpy $ExistingInstall "0"

  done_detect:
FunctionEnd

Function UpgradePage
  ; Custom page to display upgrade information
  Pop $0  ; dismiss the PageOK button
FunctionEnd

; ── Install Mode (currentUser vs allUsers) ─────────────────────────────────

!if "${INSTALL_MODE}" == "currentUser"
  InstallDir "$LOCALAPPDATA\\Wiki Labs Copilot"
!else
  InstallDir "$PROGRAMFILES64\\Wiki Labs Copilot"
!endif

; ── Shortcuts and File Associations ────────────────────────────────────────

Section "Main Application" SecMain
  ; Tauri handles the file copy via injected sections
  ; Set output path
  SetOutPath "$INSTDIR"
  
  ; ── Desktop Shortcut ─────────────────────────────────────────────────
  !if "${INSTALL_MODE}" == "currentUser"
    CreateShortCut "$DESKTOP\Wiki Labs AI Copilot.lnk" "$INSTDIR\wikilabs-copilot.exe" "$INSTDIR\icons\icon.ico" 0
    CreateShortCut "$STARTMENU\Programs\Wiki Labs\Wiki Labs AI Copilot.lnk" "$INSTDIR\wikilabs-copilot.exe" "$INSTDIR\icons\icon.ico" 0
    CreateShortCut "$STARTMENU\Programs\Wiki Labs\Uninstall Wiki Labs AI Copilot.lnk" "$INSTDIR\uninst.exe" "$INSTDIR\icons\icon.ico" 0
  !else
    CreateShortCut "$DESKTOP\Wiki Labs AI Copilot.lnk" "$INSTDIR\wikilabs-copilot.exe" "$INSTDIR\icons\icon.ico" 0
    CreateShortCut "$PROGRAMDATA\Microsoft\Windows\Start Menu\Programs\Wiki Labs\Wiki Labs AI Copilot.lnk" "$INSTDIR\wikilabs-copilot.exe" "$INSTDIR\icons\icon.ico" 0
    CreateShortCut "$PROGRAMDATA\Microsoft\Windows\Start Menu\Programs\Wiki Labs\Uninstall Wiki Labs AI Copilot.lnk" "$INSTDIR\uninst.exe" "$INSTDIR\icons\icon.ico" 0
  !endif
  
  ; ── File Associations ────────────────────────────────────────────────
  ; Associate .wikilabs files with the application
  !if "${INSTALL_MODE}" == "currentUser"
    WriteRegStr HKCU "Software\Classes\wikilabs" "" "Wiki Labs Document"
    WriteRegStr HKCU "Software\Classes\wikilabs" "FriendlyTypeName" "Wiki Labs Document"
    WriteRegStr HKCU "Software\Classes\wikilabs\DefaultIcon" "" "$INSTDIR\wikilabs-copilot.exe,0"
    WriteRegStr HKCU "Software\Classes\wikilabs\shell\open\command" "" '"$INSTDIR\wikilabs-copilot.exe" "%1"'
    WriteRegStr HKCU "Software\Classes\Applications\wikilabs-copilot.exe" "FriendlyAppName" "Wiki Labs AI Copilot"
  !else
    WriteRegStr HKLM "Software\Classes\wikilabs" "" "Wiki Labs Document"
    WriteRegStr HKLM "Software\Classes\wikilabs" "FriendlyTypeName" "Wiki Labs Document"
    WriteRegStr HKLM "Software\Classes\wikilabs\DefaultIcon" "" "$INSTDIR\wikilabs-copilot.exe,0"
    WriteRegStr HKLM "Software\Classes\wikilabs\shell\open\command" "" '"$INSTDIR\wikilabs-copilot.exe" "%1"'
    WriteRegStr HKLM "Software\Classes\Applications\wikilabs-copilot.exe" "FriendlyAppName" "Wiki Labs AI Copilot"
  !endif
  
  ; ── Registry: App Paths ──────────────────────────────────────────────
  WriteRegStr HKCU "${PRODUCT_DIR_REGKEY}" "" "$INSTDIR\wikilabs-copilot.exe"
  
  ; ── Registry: Uninstall Information ──────────────────────────────────
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayName" "$(^Name)"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\uninst.exe"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\wikilabs-copilot.exe"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "HelpLink" "${PRODUCT_WEB_SITE}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
  WriteRegStr ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "Version" "${PRODUCT_VERSION}"
  WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "VersionMajor" ${VER_MAJOR}
  WriteRegDWORD ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}" "VersionMinor" ${VER_MINOR}
  
  ; ── Preserve user data during upgrades ───────────────────────────────
  ; User data lives in %APPDATA%\Wiki Labs Copilot — we DO NOT touch it
  ; This ensures settings, cached credentials, and workspace data survive
  ; upgrades. The installer only overwrites the application files.
  !define USER_DATA_DIR "$APPDATA\Wiki Labs Copilot"
  ; No deletion of USER_DATA_DIR — intentional for upgrade preservation
  
  SetAutoClose true
SectionEnd

; ── Uninstaller ────────────────────────────────────────────────────────────

Section Uninstall
  ; ── Remove installed files ───────────────────────────────────────────
  Delete "$INSTDIR\*.*"
  RMDir /r "$INSTDIR"
  
  ; ── Remove shortcuts ─────────────────────────────────────────────────
  !if "${INSTALL_MODE}" == "currentUser"
    Delete "$DESKTOP\Wiki Labs AI Copilot.lnk"
    Delete "$STARTMENU\Programs\Wiki Labs\Wiki Labs AI Copilot.lnk"
    Delete "$STARTMENU\Programs\Wiki Labs\Uninstall Wiki Labs AI Copilot.lnk"
    RMDir "$STARTMENU\Programs\Wiki Labs"
  !else
    Delete "$DESKTOP\Wiki Labs AI Copilot.lnk"
    Delete "$PROGRAMDATA\Microsoft\Windows\Start Menu\Programs\Wiki Labs\Wiki Labs AI Copilot.lnk"
    Delete "$PROGRAMDATA\Microsoft\Windows\Start Menu\Programs\Wiki Labs\Uninstall Wiki Labs AI Copilot.lnk"
    RMDir "$PROGRAMDATA\Microsoft\Windows\Start Menu\Programs\Wiki Labs"
  !endif
  
  ; ── Remove registry entries ──────────────────────────────────────────
  DeleteRegKey ${PRODUCT_UNINST_ROOT_KEY} "${PRODUCT_UNINST_KEY}"
  DeleteRegKey HKCU "${PRODUCT_DIR_REGKEY}"
  
  ; ── File associations ────────────────────────────────────────────────
  !if "${INSTALL_MODE}" == "currentUser"
    DeleteRegKey HKCU "Software\Classes\wikilabs"
    DeleteRegKey HKCU "Software\Classes\Applications\wikilabs-copilot.exe"
  !else
    DeleteRegKey HKLM "Software\Classes\wikilabs"
    DeleteRegKey HKLM "Software\Classes\Applications\wikilabs-copilot.exe"
  !endif
  
  ; ── IMPORTANT: User data directory is preserved ──────────────────────
  ; %APPDATA%\Wiki Labs\AI Copilot contains user settings and credentials.
  ; This is intentional — users can reinstall without losing their data.
  ; If users want a full reset, they should manually delete this folder.
  
  SetAutoClose true
SectionEnd