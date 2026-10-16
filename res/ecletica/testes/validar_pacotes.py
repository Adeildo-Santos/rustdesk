#!/usr/bin/env python3
"""Testes de regressao dos pacotes gerados (instalador Windows e DMG do Mac).

Abre os arquivos de um Release e confere o que o usuario recebe. Precisa de 7-Zip
(7zz ou 7z), msitools (msiinfo), pefile (pip) e, para o DMG, de macOS (hdiutil):

    python3 res/ecletica/testes/validar_pacotes.py --versao 1.5.26 --pasta ./pacotes \\
        [--repo Adeildo-Santos/rustdesk] [--plataformas windows,mac]

--pasta deve ter os arquivos do Release ecletica-<versao>. Com --repo, confere tambem
os arquivos que o atualizador baixa da release <versao>.
Sai com codigo 1 se alguma verificacao falhar.
"""
import argparse
import plistlib
import shutil
import subprocess
import sys
import tempfile
import urllib.request
from pathlib import Path

import pefile

falhas = []
total = 0


def checar(nome, condicao, detalhe=""):
    global total
    total += 1
    if condicao:
        print(f"  ok    {nome}")
    else:
        falhas.append(nome)
        print(f"  FALHA {nome}" + (f" -- {detalhe}" if detalhe else ""))


def rodar(*cmd):
    return subprocess.run(cmd, capture_output=True, text=True, check=False).stdout


def versao_pe(caminho):
    pe = pefile.PE(str(caminho), fast_load=True)
    pe.parse_data_directories(
        directories=[pefile.DIRECTORY_ENTRY["IMAGE_DIRECTORY_ENTRY_RESOURCE"]])
    for info in getattr(pe, "FileInfo", None) or []:
        for item in info:
            for tabela in getattr(item, "StringTable", []):
                return {k.decode(): v.decode(errors="replace") for k, v in tabela.entries.items()}
    return {}


def extrair(arquivo, destino):
    sete = shutil.which("7zz") or shutil.which("7z")
    subprocess.run([sete, "x", "-y", f"-o{destino}", str(arquivo)],
                   capture_output=True, check=True)


def tabela_msi(msi, tabela):
    return rodar("msiinfo", "export", str(msi), tabela)


def validar_windows(pasta, versao, tmp):
    print("Instalador unico do Windows")
    setup = pasta / "Ecletica-Connect-Setup-Windows.exe"
    checar("Ecletica-Connect-Setup-Windows.exe existe", setup.is_file())
    if not setup.is_file():
        return
    meta = versao_pe(setup)
    checar("versao do instalador", meta.get("FileVersion") == versao, meta.get("FileVersion", ""))
    checar("copyright do instalador credita o RustDesk e cita a AGPL",
           "Purslane" in meta.get("LegalCopyright", "") and "AGPL" in meta.get("LegalCopyright", ""))
    checar("sem acentos quebrados nos metadados", "Ã" not in "".join(meta.values()))
    extrair(setup, tmp / "setup")
    plug = tmp / "setup" / "$PLUGINSDIR"
    msi = plug / "Ecletica-Connect-Windows-x64.msi"
    x86 = plug / "Ecletica-Connect-Windows-x86.exe"
    checar("instalador traz o MSI 64 bits e o cliente 32 bits", msi.is_file() and x86.is_file())
    if x86.is_file():
        checar("cliente 32 bits na mesma versao", versao_pe(x86).get("FileVersion") == versao)
    if not msi.is_file():
        return

    print("MSI 64 bits")
    props = dict(linha.split("\t", 1) for linha in tabela_msi(msi, "Property").splitlines()[3:] if "\t" in linha)
    checar("ProductName Ecletica Connect", props.get("ProductName") == "Ecletica Connect", props.get("ProductName", ""))
    checar("versao do MSI", props.get("ProductVersion", "").startswith(versao + "."), props.get("ProductVersion", ""))
    checar("UpgradeCode fixo (atualiza por cima)",
           props.get("UpgradeCode") == "{D3DC41E3-3C37-5D52-BB47-F6673705C01B}", props.get("UpgradeCode", ""))
    checar("links de suporte sem o GitHub do RustDesk",
           not any("github.com/rustdesk" in v for v in props.values()))
    arquivos = tabela_msi(msi, "File")
    for nome in ["Ecletica Connect.exe", "LICENCE.txt", "AVISOS-LEGAIS.txt"]:
        checar(f"MSI instala {nome}", f"|{nome}\t" in arquivos or f"\t{nome}\t" in arquivos)
    acoes = tabela_msi(msi, "CustomAction")
    checar("servico 'Ecletica Connect' criado pelo MSI",
           'Ecletica Connect;"[INSTALLFOLDER_INNER]Ecletica Connect.exe" --service' in acoes)
    checar("servico antigo 'Ecletica Acesso Remoto' removido", "Ecletica Acesso Remoto" in acoes)
    checar("processo auxiliar RuntimeBroker_ecletica.exe", "RuntimeBroker_ecletica.exe" in acoes)
    controles = tabela_msi(msi, "Control")
    checar("tela de licenca com avisos e AGPL", "AVISOS LEGAIS" in controles and "AFFERO" in controles)


def validar_mac(pasta, versao, tmp):
    for dmg_nome, arch in [("Ecletica-Connect-macOS-AppleSilicon.dmg", "arm64"),
                           ("Ecletica-Connect-macOS-Intel.dmg", "x86_64")]:
        print(f"Mac: {dmg_nome}")
        dmg = pasta / dmg_nome
        checar(f"{dmg_nome} existe", dmg.is_file())
        if not dmg.is_file():
            continue
        if not shutil.which("hdiutil"):
            print("  (pulado: precisa de macOS para abrir o DMG)")
            continue
        vol = tmp / f"vol-{arch}"
        subprocess.run(["hdiutil", "attach", "-nobrowse", "-readonly", "-mountpoint", str(vol), str(dmg)],
                       capture_output=True, check=True)
        try:
            app = vol / "Ecletica Connect.app"
            info = plistlib.loads((app / "Contents/Info.plist").read_bytes())
            checar("nome exibido Eclética Connect", info.get("CFBundleDisplayName") == "Eclética Connect"
                   and info.get("CFBundleName") == "Eclética Connect", info.get("CFBundleDisplayName", ""))
            checar("identificador br.com.ecletica.connect",
                   info.get("CFBundleIdentifier") == "br.com.ecletica.connect", info.get("CFBundleIdentifier", ""))
            checar("versao do app", info.get("CFBundleShortVersionString") == versao,
                   info.get("CFBundleShortVersionString", ""))
            checar("copyright cita a AGPL", "AGPL" in info.get("NSHumanReadableCopyright", ""))
            recursos = app / "Contents/Resources"
            checar("licenca e avisos dentro do app",
                   (recursos / "LICENCE.txt").is_file() and (recursos / "AVISOS-LEGAIS.txt").is_file())
            binario = app / "Contents/MacOS/Ecletica Connect"
            checar(f"arquitetura {arch}", arch in rodar("lipo", "-archs", str(binario)))
            assinatura = subprocess.run(["codesign", "--verify", "--deep", "--strict", str(app)],
                                        capture_output=True, text=True)
            checar("assinatura valida (Acessibilidade honra os cliques)", assinatura.returncode == 0,
                   assinatura.stderr.strip())
            app_flutter = app / "Contents/Frameworks/App.framework/Versions/A/App"
            checar("Sobre com credito ao RustDesk e AGPL", b"baseado no RustDesk" in app_flutter.read_bytes())
        finally:
            subprocess.run(["hdiutil", "detach", str(vol)], capture_output=True)


def validar_atualizador(repo, versao, plataformas):
    print("Arquivos do atualizador (release da versao)")
    nomes = []
    if "windows" in plataformas:
        nomes += [f"rustdesk-{versao}-x86_64.msi", f"rustdesk-{versao}-x86_64.exe",
                  f"rustdesk-{versao}-x86-sciter.exe"]
    if "mac" in plataformas:
        nomes += [f"rustdesk-{versao}-aarch64.dmg", f"rustdesk-{versao}-x86_64.dmg"]
    for nome in nomes:
        url = f"https://github.com/{repo}/releases/download/{versao}/{nome}"
        try:
            req = urllib.request.Request(url, method="HEAD")
            ok = urllib.request.urlopen(req, timeout=30).status == 200
        except Exception:
            ok = False
        checar(f"{nome} publicado", ok)


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--versao", required=True)
    p.add_argument("--pasta", required=True, type=Path)
    p.add_argument("--repo")
    p.add_argument("--plataformas", default="windows,mac")
    a = p.parse_args()
    plataformas = a.plataformas.split(",")
    with tempfile.TemporaryDirectory() as tmp:
        if "windows" in plataformas:
            validar_windows(a.pasta, a.versao, Path(tmp))
        if "mac" in plataformas:
            validar_mac(a.pasta, a.versao, Path(tmp))
    if a.repo:
        validar_atualizador(a.repo, a.versao, plataformas)
    print()
    if falhas:
        print(f"{len(falhas)} de {total} verificacoes falharam.")
        sys.exit(1)
    print(f"Todas as {total} verificacoes passaram.")


if __name__ == "__main__":
    main()
