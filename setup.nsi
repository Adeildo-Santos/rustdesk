Unicode true

####################################################################
# Ecletica Connect - instalador do cliente 32 bits (arvore sciter 1.1.8)
# Este instalador e para teste/entrega direta. O instalador unico da
# Ecletica (Ecletica-Connect-Setup-Windows.exe) tambem usa este cliente.

!include nsDialogs.nsh
!include MUI2.nsh
!include LogicLib.nsh

####################################################################
# File Info

!define PRODUCT_NAME "Ecletica Connect"
!define PRODUCT_DESCRIPTION "Instalador do Ecletica Connect (32 bits)"
!define COPYRIGHT "Copyright © Ecletica"
!define VERSION "1.5.0"

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "${PRODUCT_NAME}"
VIAddVersionKey "ProductVersion" "${VERSION}"
VIAddVersionKey "FileDescription" "${PRODUCT_DESCRIPTION}"
VIAddVersionKey "LegalCopyright" "${COPYRIGHT}"
VIAddVersionKey "FileVersion" "${VERSION}.0"

####################################################################
# Installer Attributes

Name "${PRODUCT_NAME}"
Outfile "Ecletica-Connect-32bits-Setup.exe"
Caption "Instalacao - ${PRODUCT_NAME}"
BrandingText "${PRODUCT_NAME}"

ShowInstDetails show
RequestExecutionLevel admin
SetOverwrite on

InstallDir "$PROGRAMFILES\${PRODUCT_NAME}"

####################################################################
# Pages

!define MUI_ICON "src\tray-icon.ico"
!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_LANGUAGE "PortugueseBR"

####################################################################
# Sections

Section "Install"
  SetOutPath $INSTDIR

  # Regkeys (nome, fabricante e enderecos da Ecletica)
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" "DisplayIcon" "$INSTDIR\${PRODUCT_NAME}.exe"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" "DisplayName" "${PRODUCT_NAME} (32 bits)"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" "UninstallString" '"$INSTDIR\${PRODUCT_NAME}.exe" --uninstall'
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" "Publisher" "Ecletica"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" "HelpLink" "https://app.ecletico.ai"
  WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}" "URLInfoAbout" "https://app.ecletico.ai"

  nsExec::Exec 'taskkill /F /IM "${PRODUCT_NAME}.exe"'
  Sleep 500
  File "${PRODUCT_NAME}.exe"
  ; a interface e a biblioteca do sciter ficam ao lado do executavel
  File "sciter.dll"
  File "icon.ico"
  File /r /x "*.rs" "src\ui\*.*"

  SetShellVarContext all
  CreateDirectory "$SMPROGRAMS\${PRODUCT_NAME}"
  CreateShortCut "$DESKTOP\${PRODUCT_NAME}.lnk" "$INSTDIR\${PRODUCT_NAME}.exe"
  CreateShortCut "$SMPROGRAMS\${PRODUCT_NAME}\${PRODUCT_NAME}.lnk" "$INSTDIR\${PRODUCT_NAME}.exe"
  CreateShortCut "$SMPROGRAMS\${PRODUCT_NAME}\Uninstall ${PRODUCT_NAME}.lnk" "$INSTDIR\${PRODUCT_NAME}.exe" "--uninstall" "msiexec.exe"
  CreateShortCut "$SMSTARTUP\${PRODUCT_NAME} Tray.lnk" "$INSTDIR\${PRODUCT_NAME}.exe" "--tray"

  nsExec::Exec 'sc create "${PRODUCT_NAME}" start=auto DisplayName="${PRODUCT_NAME} Service" binPath= "\"$INSTDIR\${PRODUCT_NAME}.exe\" --service"'
  nsExec::Exec 'netsh advfirewall firewall add rule name="${PRODUCT_NAME} Service" dir=in action=allow program="$INSTDIR\${PRODUCT_NAME}.exe" enable=yes'
  nsExec::Exec 'sc start "${PRODUCT_NAME}"'
SectionEnd

####################################################################
# Functions

Function .onInit
  ${IfNot} ${AtLeastWin7}
    MessageBox MB_OK "Este instalador requer Windows 7 ou superior."
    Abort
  ${EndIf}
FunctionEnd
