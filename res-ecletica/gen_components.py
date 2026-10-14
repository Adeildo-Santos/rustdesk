#!/usr/bin/env python3
"""Gera Payload.wxs a partir de res-ecletica/payload/{x64,x86}.

Cada arquivo vira um Component com Condition por arquitetura:
  x64 -> VersionNT64        (cliente Flutter)
  x86 -> NOT VersionNT64    (cliente sciter legado)
O exe principal de cada arquitetura ("Ecletica Connect.exe") recebe tambem o
servico (ServiceInstall/ServiceControl) e a regra de firewall.
Tudo instala em INSTALLFOLDER, espelhando a arvore de pastas de cada payload.
"""
import os
import sys
import uuid

HERE = os.path.dirname(os.path.abspath(__file__))
PAYLOAD = os.path.join(HERE, "payload")
EXE = "Ecletica Connect.exe"
NS = "http://wixtoolset.org/schemas/v4/wxs"
FW = "http://wixtoolset.org/schemas/v4/wxs/firewall"


def rid(prefix, path):
    s = path.replace("\\", "/").strip("/")
    out = "".join(c if c.isalnum() else "_" for c in s)
    full = f"{prefix}_{out}" if out else prefix
    # Ids do WiX devem ter <= 72 chars; encurta mantendo unicidade por hash.
    if len(full) > 70:
        import hashlib
        h = hashlib.sha1(path.encode("utf-8")).hexdigest()[:8]
        full = f"{prefix}_{out[:50]}_{h}"
    return full


def esc(s):
    return s.replace("&", "&amp;").replace("<", "&lt;").replace(">", "&gt;")


def collect(arch):
    """Lista (relpath_dir, filename) de todos os arquivos do payload/<arch>."""
    root = os.path.join(PAYLOAD, arch)
    files = []
    for dirpath, _dirs, names in os.walk(root):
        rel = os.path.relpath(dirpath, root)
        rel = "" if rel == "." else rel.replace("/", "\\")
        for n in sorted(names):
            files.append((rel, n))
    return sorted(files)


def main():
    lines = []
    lines.append('<?xml version="1.0" encoding="utf-8"?>')
    lines.append(f'<Wix xmlns="{NS}" xmlns:fw="{FW}">')
    lines.append("  <Fragment>")

    # Arvore de diretorios (uniao das duas arquiteturas) sob INSTALLFOLDER.
    dirs = {}  # relpath -> dir id  ("" = INSTALLFOLDER)
    dirs[""] = "INSTALLFOLDER"
    for arch in ("x64", "x86"):
        for rel, _n in collect(arch):
            parts = rel.split("\\") if rel else []
            acc = ""
            for p in parts:
                parent = acc
                acc = (acc + "\\" + p) if acc else p
                if acc not in dirs:
                    dirs[acc] = rid("dir", acc)

    # Emite a arvore aninhada a partir de INSTALLFOLDER.
    children = {}
    for rel, did in dirs.items():
        if rel == "":
            continue
        parent = "\\".join(rel.split("\\")[:-1])
        children.setdefault(parent, []).append((rel, did))

    def emit_dir(rel, indent):
        for crel, cid in sorted(children.get(rel, [])):
            name = crel.split("\\")[-1]
            lines.append(f'{indent}<Directory Id="{cid}" Name="{esc(name)}">')
            emit_dir(crel, indent + "  ")
            lines.append(f"{indent}</Directory>")

    lines.append('    <DirectoryRef Id="INSTALLFOLDER">')
    emit_dir("", "      ")
    lines.append("    </DirectoryRef>")

    # Componentes.
    lines.append('    <ComponentGroup Id="Payload">')
    for arch, cond in (("x64", "VersionNT64"), ("x86", "NOT VersionNT64")):
        for rel, n in collect(arch):
            did = dirs[rel]
            src = os.path.join("payload", arch, *( [rel] if rel else [] ), n).replace("/", "\\")
            cid = rid(f"c_{arch}", (rel + "\\" + n) if rel else n)
            fid = rid(f"f_{arch}", (rel + "\\" + n) if rel else n)
            is_exe = (rel == "" and n == EXE)
            lines.append(f'      <Component Id="{cid}" Directory="{did}" Guid="*" Condition="{esc(cond)}">')
            if is_exe:
                lines.append(f'        <File Id="{fid}" Source="{esc(src)}" KeyPath="yes">')
                lines.append(f'          <fw:FirewallException Id="Fw_{arch}" Name="Ecletica Connect" Scope="any" IgnoreFailure="yes" />')
                lines.append("        </File>")
                lines.append(f'        <ServiceInstall Id="Svc_{arch}" Name="Ecletica Connect" DisplayName="Ecletica Connect Service" Start="auto" Type="ownProcess" ErrorControl="normal" Arguments="--service" Account="LocalSystem" Vital="no" />')
                lines.append(f'        <ServiceControl Id="SvcCtrl_{arch}" Name="Ecletica Connect" Start="install" Stop="both" Remove="uninstall" Wait="no" />')
            else:
                lines.append(f'        <File Id="{fid}" Source="{esc(src)}" KeyPath="yes" />')
            lines.append("      </Component>")
    lines.append("    </ComponentGroup>")
    lines.append("  </Fragment>")
    lines.append("</Wix>")

    out = os.path.join(HERE, "Payload.wxs")
    with open(out, "w", encoding="utf-8") as f:
        f.write("\n".join(lines) + "\n")
    nx64 = len(collect("x64"))
    nx86 = len(collect("x86"))
    print(f"Payload.wxs gerado: {nx64} arquivos x64 + {nx86} arquivos x86, {len(dirs)} diretorios")


if __name__ == "__main__":
    sys.exit(main())
