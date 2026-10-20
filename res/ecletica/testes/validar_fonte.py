#!/usr/bin/env python3
"""Testes de regressao do codigo-fonte do Eclética Connect.

Cada verificacao trava um bug ja corrigido (ou uma regra de licenca/marca) para que
ele nao volte. Roda em segundos, sem compilar nada:

    python3 res/ecletica/testes/validar_fonte.py

Ao corrigir um bug novo, acrescente uma verificacao aqui (ou em validar_pacotes.py,
se o defeito so aparece no instalador/app gerado) com a data e o sintoma.
Sai com codigo 1 se alguma verificacao falhar.
"""
import hashlib
import re
import sys
from pathlib import Path

RAIZ = Path(__file__).resolve().parents[3]
WORKFLOW = ".github/workflows/flutter-build.yml"
falhas = []
total = 0


def ler(caminho):
    return (RAIZ / caminho).read_text(encoding="utf-8-sig")


def checar(nome, condicao, detalhe=""):
    global total
    total += 1
    if condicao:
        print(f"  ok    {nome}")
    else:
        falhas.append(nome)
        print(f"  FALHA {nome}" + (f" -- {detalhe}" if detalhe else ""))


def contem(caminho, trecho, vezes=None):
    n = ler(caminho).count(trecho)
    return n == vezes if vezes is not None else n > 0


def sha(caminho):
    return hashlib.sha1((RAIZ / caminho).read_bytes()).hexdigest()[:12]


print("Versao")
versoes = {
    "Cargo.toml": re.search(r'^version = "([^"]+)"', ler("Cargo.toml"), re.M).group(1),
    "Cargo.lock (rustdesk)": re.search(r'name = "rustdesk"\nversion = "([^"]+)"', ler("Cargo.lock")).group(1),
    "Cargo.lock (packer)": re.search(r'name = "rustdesk-portable-packer"\nversion = "([^"]+)"', ler("Cargo.lock")).group(1),
    "libs/portable/Cargo.toml": re.search(r'^version = "([^"]+)"', ler("libs/portable/Cargo.toml"), re.M).group(1),
    "flutter/pubspec.yaml": re.search(r"^version: ([0-9.]+)\+", ler("flutter/pubspec.yaml"), re.M).group(1),
    WORKFLOW: re.search(r'^  VERSION: "([^"]+)"', ler(WORKFLOW), re.M).group(1),
}
checar("mesma versao em Cargo, lock, packer, pubspec e workflow (bug: 1.5.0+68 x 1.5.20)",
       len(set(versoes.values())) == 1, str(versoes))

print("Licenca AGPL e avisos")
checar("LICENCE e AVISOS-LEGAIS.txt existem",
       (RAIZ / "LICENCE").is_file() and (RAIZ / "AVISOS-LEGAIS.txt").is_file())
checar("LICENCE e a AGPL-3.0", contem("LICENCE", "GNU AFFERO GENERAL PUBLIC LICENSE"))
for f in ["Cargo.toml", "libs/portable/Cargo.toml", "flutter/windows/runner/Runner.rc",
          "flutter/macos/Runner/Configs/AppInfo.xcconfig", "res/ecletica/instalador.nsi", WORKFLOW]:
    t = ler(f)
    checar(f"sem 'Todos os direitos reservados'/'All rights reserved' em {f}",
           "Todos os direitos reservados" not in t and "All rights reserved" not in t)
checar("tela de licenca do MSI e a dos avisos + AGPL (bug: mostrava a politica de privacidade da Purslane)",
       contem("res/msi/Package/License.rtf", "AVISOS LEGAIS") and contem("res/msi/Package/License.rtf", "AFFERO")
       and not contem("res/msi/Package/License.rtf", "Privacy Policy"))
checar("licenca e avisos copiados para o programa (64, 32 bits e Mac)",
       contem(WORKFLOW, "cp LICENCE ./rustdesk/LICENCE.txt") and contem(WORKFLOW, "cp LICENCE ./Release/LICENCE.txt")
       and contem(WORKFLOW, 'cp LICENCE "$APP/Contents/Resources/LICENCE.txt"'))
checar("tag da versao aponta para o commit compilado (codigo-fonte correspondente)",
       ler(WORKFLOW).count("target_commitish: ${{ github.sha }}") >= 3)

print("Sobre (avisos legais na interface)")
checar("Sobre do 64 bits/Mac: credito ao RustDesk, AGPL e link do codigo-fonte",
       contem("flutter/lib/desktop/pages/desktop_setting_page.dart", "baseado no RustDesk")
       and contem("flutter/lib/desktop/pages/desktop_setting_page.dart", "Adeildo-Santos/rustdesk/releases"))
checar("Sobre do 32 bits: credito, AGPL, codigo-fonte e atribuicao do Sciter",
       contem("src/ui/index.tis", "baseado no RustDesk") and contem("src/ui/index.tis", "uses Sciter Engine")
       and contem("src/ui/index.tis", "Adeildo-Santos/rustdesk/releases"))
checar("Sobre do Android: codigo-fonte e credito",
       contem("flutter/lib/mobile/pages/settings_page.dart", "Baseado no RustDesk"))

print("Marca")
for f in ["Cargo.toml", "libs/portable/Cargo.toml"]:
    checar(f"metadados do executavel com a marca em {f} (bug: 32 bits dizia ProductName=RustDesk)",
           contem(f, 'ProductName = "Eclética Connect"') and contem(f, 'OriginalFilename = "Ecletica Connect.exe"'))
checar("logo do RustDesk fora da interface do 32 bits (src/ui.rs)",
       "WXMAAEiuAABIrgHwmhA7AAAAGXRFWHRTb2Z0d2FyZQB3d3cuaW5rc2NhcGUub" not in ler("src/ui.rs"))
checar("icon.svg reserva nao e o logo do RustDesk", sha("flutter/assets/icon.svg") != "3803cf698b32")
checar("icone de notificacao do Android nao e o do RustDesk",
       sha("flutter/android/app/src/main/res/mipmap-xxxhdpi/ic_stat_logo.png") != "8315ddb592e6")
checar("icone da bandeja do Mac nao e o do RustDesk", sha("res/mac-tray-dark-x2.png") != "fef4ca3c8499")
checar("logo ECLÉTICA na tela principal do 32 bits",
       contem("src/ui/index.css", "#ecletica-logo") and contem("src/ui/index.tis", "<div #ecletica-logo />"))
checar("links de privacidade/download sem rustdesk.com no 32 bits",
       "rustdesk.com/privacy" not in ler("src/ui/index.tis") + ler("src/ui/install.tis")
       and "rustdesk.com/download" not in ler("src/ui/index.tis"))
checar("'Adicionar/remover programas' sem links do GitHub do RustDesk",
       "github.com/rustdesk/rustdesk" not in ler("res/msi/preprocess.py").split("def default_revision_version")[0])
checar("'Desenvolvido por RustDesk' oculto (hide-powered-by-me ligado no codigo)",
       contem("src/common.rs", "keys::OPTION_HIDE_POWERED_BY_ME.to_owned(), \"Y\"")
       and contem("src/common.rs", "    apply_ecletica_settings();"))
checar("processo auxiliar sem o nome RustDesk (RuntimeBroker_ecletica.exe)",
       contem("src/privacy_mode/win_topmost_window.rs", '"RuntimeBroker_ecletica.exe"')
       and contem("libs/portable/src/main.rs", '"RuntimeBroker_ecletica.exe"', 2)
       and contem("res/msi/Package/Components/RustDesk.wxs", 'Value="RuntimeBroker_ecletica.exe"'))
checar("Android com identificador proprio",
       contem("flutter/android/app/build.gradle", 'applicationId "br.com.ecletica.connect"'))
checar("Mac com identificador proprio e nome Eclética Connect",
       contem("flutter/macos/Runner.xcodeproj/project.pbxproj", "PRODUCT_BUNDLE_IDENTIFIER = br.com.ecletica.connect;", 3)
       and contem("flutter/macos/Runner/Configs/AppInfo.xcconfig", "PRODUCT_BUNDLE_IDENTIFIER = br.com.ecletica.connect")
       and contem("flutter/macos/Runner/Info.plist", "<string>Eclética Connect</string>", 2)
       and "Acesso Remoto" not in ler("flutter/macos/Runner/Info.plist"))

print("Instalacao e atualizacao no Windows")
checar("32 bits empacotado como 'Ecletica Connect.exe' (bug: instalava rustdesk.exe ao lado do antigo)",
       contem(WORKFLOW, 'mv ./target/release/rustdesk.exe "./Release/Ecletica Connect.exe"')
       and contem(WORKFLOW, '-e "../../Release/Ecletica Connect.exe"'))
checar("instalador unico apaga o rustdesk.exe deixado pela 1.5.24",
       contem("res/ecletica/instalador.nsi", 'Delete "$PROGRAMFILES\\Ecletica Connect\\rustdesk.exe"'))
checar("instalador unico escolhe 64/32 bits", contem("res/ecletica/instalador.nsi", "${RunningX64}"))
checar("instalador unico silencioso (bug: passou a pedir cliques e mostrar telas)",
       contem("res/ecletica/instalador.nsi", "SilentInstall silent")
       and contem("res/ecletica/instalador.nsi", "/qn /norestart")
       and "MUI_PAGE" not in ler("res/ecletica/instalador.nsi"))
checar("instalador so 64 bits avisa no Windows 32 bits (bug: erro 1633)",
       contem(WORKFLOW, "${IsNativeIA32}"))
checar("acentos corretos nos instaladores NSIS (BOM UTF-8; bug: 'EclÃ©tica')",
       (RAIZ / "res/ecletica/instalador.nsi").read_bytes()[:3] == b"\xef\xbb\xbf"
       and contem(WORKFLOW, "printf '\\xEF\\xBB\\xBF' > instalador.nsi"))
checar("32 bits se atualiza pelo executavel, nao pelo MSI",
       contem("src/updater.rs", 'let update_msi = update_msi && cfg!(feature = "flutter");'))
checar("arquivo do 32 bits publicado na release da versao (atualizador)",
       contem(WORKFLOW, "files: ./SignOutput/rustdesk-${{ env.VERSION }}-x86-sciter.exe"))
win = ler("src/platform/windows.rs")
checar("registro de 'Adicionar ou remover programas' com aspas (bug: 32 bits nao saia da lista ao desinstalar)",
       "reg add {subkey}" not in win and "reg delete {subkey}" not in win
       and 'reg delete \\"{subkey}\\" /f' in win and win.count('reg add \\"{subkey}\\" /f') >= 21)
checar("desinstalacao espera os processos fecharem antes de apagar a pasta (bug: sobravam arquivos)",
       'ping -n 4 127.0.0.1 > nul\n    if exist \\"{path}\\" rd /s /q' in win)
checar("pasta de dados congelada (guarda ID e senha das maquinas)",
       contem("libs/hbb_common/src/config.rs", 'pub const DATA_DIR_NAME: &str = "Ecletica Acesso Remoto";'))
checar("UpgradeCode do MSI congelado (atualiza por cima, sem instalar duplicado)",
       contem("res/msi/preprocess.py", 'UPGRADE_CODE_SEED = "RustDesk.exe"'))

checar("build publica no Release 'ecletica', lido pelo servidor do rustdesk-versao.json",
       contem(".github/workflows/ecletica-build.yml", "upload-tag: ${{ github.event.inputs.tag || 'ecletica' }}")
       and contem(WORKFLOW, "./SignOutput/VERSION.txt"))

checar("cliente informa a arquitetura no sysinfo (painel conta as maquinas 32 bits)",
       contem("src/common.rs", 'os = format!("{os} - {}", ecletica_arquitetura());')
       and contem("src/common.rs", '"32 bits"'))

checar("Release 'ecletica' passa a apontar para o commit compilado a cada versao (bug: ficou na 1.5.27)",
       contem(WORKFLOW, 'gh api -X PATCH "repos/$R/git/refs/tags/$T" -f sha="${{ github.sha }}" -F force=true'))

checar("bandeja do Windows com icone proprio (monitores com setas), sem usar o icon.png do app",
       contem("src/tray.rs", 'icon = include_bytes!("../res/tray-icon-windows.png");')
       and contem("src/tray.rs", "let asset = None::<image::DynamicImage>;")
       and (RAIZ / "res/tray-icon-windows.png").is_file())

checar("Release 'ecletica' recebe arquivos pelo gh, nao pelo action-gh-release (bug: 'Resource not accessible by integration')",
       ler(WORKFLOW).count('publicar_no_release.sh "${{ env.TAG_NAME }}"') == 4
       and (RAIZ / "res/ecletica/publicar_no_release.sh").is_file())

print("Sessao remota")
checar("'Somente visualizar' desligado por padrao (bug: Mac nao clicava nem digitava)",
       contem("src/common.rs", "keys::OPTION_VIEW_ONLY.to_owned(), \"N\""))
checar("'Somente visualizar' zerado ao abrir cada sessao",
       contem("src/client.rs", "if config.view_only.v {\n            config.view_only.v = false;"))
checar("Mac: trackpad tratado como mouse", contem("flutter/lib/models/input_model.dart", "_isMouseLikeKind"))
checar("Mac: reassinatura ad-hoc no fim do build (Acessibilidade honra os cliques)",
       contem(WORKFLOW, 'codesign --force --deep --sign - "$APP"'))

print()
if falhas:
    print(f"{len(falhas)} de {total} verificacoes falharam.")
    sys.exit(1)
print(f"Todas as {total} verificacoes passaram.")
