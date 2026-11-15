Unicode true
!include "MUI2.nsh"
!include "x64.nsh"

!define APP "Game of Life"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\RandelDodson.GameOfLife"
Name "${APP}"
OutFile "../../installers/Game-of-Life-${VERSION}-Windows-x64-Setup.exe"
InstallDir "$LOCALAPPDATA\Programs\Game of Life"
RequestExecutionLevel user
SetCompressor /SOLID lzma
Icon "../assets/app-icon.ico"
UninstallIcon "../assets/app-icon.ico"
VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "${APP}"
VIAddVersionKey "FileDescription" "${APP} x64 Setup"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "LegalCopyright" "© 2026 Randel Dodson"

!insertmacro MUI_PAGE_LICENSE "../../LICENSE"
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Function .onInit
    ${IfNot} ${RunningX64}
        MessageBox MB_OK "Game of Life requires 64-bit Windows."
        Abort
    ${EndIf}
FunctionEnd

Section
    SetRegView 64
    SetOutPath "$INSTDIR"
    File "/oname=Game of Life.exe" "../target/x86_64-pc-windows-gnu/release/game-of-life-bevy.exe"
    File "/oname=LICENSE.txt" "../../LICENSE"
    SetOutPath "$INSTDIR\assets\patterns"
    File "../assets/patterns/catalog.json"
    File "../assets/patterns/catalog.tsv"
    File "../assets/patterns/puffer1.rle"
    SetOutPath "$INSTDIR"
    WriteUninstaller "$INSTDIR\Uninstall.exe"
    CreateShortcut "$SMPROGRAMS\Game of Life.lnk" "$INSTDIR\Game of Life.exe"
    WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "${APP}"
    WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${VERSION}"
    WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "Randel Dodson"
    WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\Game of Life.exe"
    WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" '"$INSTDIR\Uninstall.exe"'
SectionEnd

Section "Uninstall"
    SetRegView 64
    Delete "$SMPROGRAMS\Game of Life.lnk"
    Delete "$INSTDIR\assets\patterns\puffer1.rle"
    Delete "$INSTDIR\assets\patterns\catalog.tsv"
    Delete "$INSTDIR\assets\patterns\catalog.json"
    RMDir "$INSTDIR\assets\patterns"
    RMDir "$INSTDIR\assets"
    Delete "$INSTDIR\LICENSE.txt"
    Delete "$INSTDIR\Game of Life.exe"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"
    DeleteRegKey HKCU "${UNINSTALL_KEY}"
SectionEnd
