; Instalador unico do Ecletica Connect (Windows 32 e 64 bits).
; Compilado pelo job "instalador-unico" do flutter-build.yml:
;   makensis -DVERSAO=1.2.3 -DMSI64=<msi x64> -DEXE32=<exe x86 sciter> -DSAIDA=<saida> instalador.nsi
; Mostra a licenca (AVISOS-LEGAIS + AGPL), depois instala o MSI no Windows 64 bits
; ou o cliente Sciter no Windows 32 bits.
Unicode true
!define PRODUTO "Eclética Connect"
!define EMPRESA "Eclética Tecnologia Ltda."

Name "${PRODUTO}"
OutFile "${SAIDA}"
Icon "app_icon.ico"
RequestExecutionLevel admin
SetCompressor /SOLID lzma
ShowInstDetails nevershow

VIProductVersion "${VERSAO}.0"
VIAddVersionKey "ProductName" "${PRODUTO}"
VIAddVersionKey "ProductVersion" "${VERSAO}"
VIAddVersionKey "FileDescription" "Instalador do ${PRODUTO}"
VIAddVersionKey "FileVersion" "${VERSAO}"
VIAddVersionKey "CompanyName" "${EMPRESA}"
VIAddVersionKey "LegalCopyright" "Baseado no RustDesk © Purslane Tech Pte. Ltd. e colaboradores. Modificações © 2026 Eclética Tecnologia Ltda. Licença AGPL-3.0."

!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"

!define MUI_ICON "app_icon.ico"
!define MUI_LICENSEPAGE_TEXT_TOP "Leia os avisos legais e a licença do software."
!define MUI_LICENSEPAGE_BUTTON "Instalar"
!insertmacro MUI_PAGE_LICENSE "LICENCA.txt"
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_LANGUAGE "PortugueseBR"

Section "Instalar"
  InitPluginsDir
  SetOutPath "$PLUGINSDIR"
  ${If} ${RunningX64}
    DetailPrint "Instalando a versão 64 bits..."
    File "${MSI64}"
    ExecWait '"$SYSDIR\msiexec.exe" /i "$PLUGINSDIR\${MSI64}" /passive' $0
  ${Else}
    DetailPrint "Instalando a versão 32 bits..."
    File "${EXE32}"
    ExecWait '"$PLUGINSDIR\${EXE32}" --silent-install' $0
    ; a 1.5.24 deixou um rustdesk.exe solto ao lado do Ecletica Connect.exe
    Delete "$PROGRAMFILES\Ecletica Connect\rustdesk.exe"
  ${EndIf}
  ${If} $0 == 3010
    MessageBox MB_OK "${PRODUTO} foi instalado. Reinicie o Windows para concluir."
  ${ElseIf} $0 == 1602
    Abort
  ${ElseIf} $0 != 0
    MessageBox MB_ICONSTOP "A instalação terminou com o código $0. Anote este número e avise o suporte."
    Abort
  ${EndIf}
SectionEnd
