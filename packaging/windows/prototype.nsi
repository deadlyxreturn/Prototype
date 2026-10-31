; Prototype installer (NSIS) - a Second Life viewer by Ryan Johnson.
; Built by the Release workflow:  makensis /DVERSION=1.0.0 /DSRC=<package folder> prototype.nsi
Unicode true
!include "MUI2.nsh"

!ifndef VERSION
  !define VERSION "1.0.0"
!endif
!ifndef SRC
  !define SRC "..\..\Prototype-Viewer"
!endif

Name "Prototype ${VERSION}"
OutFile "Prototype-Setup-${VERSION}.exe"
; per user: no administrator rights needed, and any folder can be chosen
InstallDir "$LOCALAPPDATA\Programs\Prototype"
InstallDirRegKey HKCU "Software\Prototype" "InstallDir"
RequestExecutionLevel user
SetCompressor /SOLID lzma
BrandingText "Prototype by Ryan Johnson"

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "Prototype"
VIAddVersionKey "CompanyName" "Ryan Johnson"
VIAddVersionKey "FileDescription" "Prototype installer"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "LegalCopyright" "Prototype by Ryan Johnson. GPLv3."

!define MUI_ICON "..\..\assets\branding\prototype.ico"
!define MUI_UNICON "..\..\assets\branding\prototype.ico"
!define MUI_ABORTWARNING
!define MUI_WELCOMEPAGE_TITLE "Install Prototype ${VERSION}"
!define MUI_WELCOMEPAGE_TEXT "Prototype is a Second Life viewer by Ryan Johnson.$\r$\n$\r$\nThis software is not provided or supported by Linden Lab, the makers of Second Life.$\r$\n$\r$\nClick Next to continue."
!define MUI_FINISHPAGE_RUN "$INSTDIR\prototype-viewer.exe"
!define MUI_FINISHPAGE_RUN_TEXT "Start Prototype"

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "Prototype" SecMain
  SectionIn RO
  SetOutPath "$INSTDIR"
  File /r "${SRC}\*.*"
  File "/oname=prototype.ico" "..\..\assets\branding\prototype.ico"
  WriteRegStr HKCU "Software\Prototype" "InstallDir" "$INSTDIR"
  WriteUninstaller "$INSTDIR\Uninstall Prototype.exe"
  CreateDirectory "$SMPROGRAMS\Prototype"
  CreateShortcut "$SMPROGRAMS\Prototype\Prototype.lnk" "$INSTDIR\prototype-viewer.exe" "" "$INSTDIR\prototype.ico"
  CreateShortcut "$SMPROGRAMS\Prototype\Uninstall Prototype.lnk" "$INSTDIR\Uninstall Prototype.exe"
  CreateShortcut "$DESKTOP\Prototype.lnk" "$INSTDIR\prototype-viewer.exe" "" "$INSTDIR\prototype.ico"
  !define UNINST "Software\Microsoft\Windows\CurrentVersion\Uninstall\Prototype"
  WriteRegStr HKCU "${UNINST}" "DisplayName" "Prototype"
  WriteRegStr HKCU "${UNINST}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINST}" "Publisher" "Ryan Johnson"
  WriteRegStr HKCU "${UNINST}" "DisplayIcon" "$INSTDIR\prototype.ico"
  WriteRegStr HKCU "${UNINST}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINST}" "URLInfoAbout" "https://www.prototypesl.com"
  WriteRegStr HKCU "${UNINST}" "UninstallString" "$\"$INSTDIR\Uninstall Prototype.exe$\""
  WriteRegDWORD HKCU "${UNINST}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINST}" "NoRepair" 1
SectionEnd

Section "Uninstall"
  Delete "$DESKTOP\Prototype.lnk"
  RMDir /r "$SMPROGRAMS\Prototype"
  RMDir /r "$INSTDIR"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\Prototype"
  DeleteRegKey HKCU "Software\Prototype"
SectionEnd
