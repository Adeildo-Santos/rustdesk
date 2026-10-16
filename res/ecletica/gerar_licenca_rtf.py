#!/usr/bin/env python3
"""Gera res/msi/Package/License.rtf (tela de licenca do MSI) a partir de
AVISOS-LEGAIS.txt e LICENCE. Rodar de novo sempre que um dos dois mudar."""
from pathlib import Path

RAIZ = Path(__file__).resolve().parents[2]


def rtf(texto):
    saida = []
    for c in texto:
        if c in "\\{}":
            saida.append("\\" + c)
        elif c == "\n":
            saida.append("\\par\n")
        elif ord(c) > 127:
            saida.append(f"\\u{ord(c)}?")
        else:
            saida.append(c)
    return "".join(saida)


def main():
    avisos = (RAIZ / "AVISOS-LEGAIS.txt").read_text(encoding="utf-8")
    licenca = (RAIZ / "LICENCE").read_text(encoding="utf-8")
    corpo = rtf(avisos) + "\\par\n\\par\n" + rtf(licenca)
    doc = (
        "{\\rtf1\\ansi\\ansicpg1252\\deff0\\uc1"
        "{\\fonttbl{\\f0\\fmodern\\fcharset0 Consolas;}}"
        "\\viewkind4\\f0\\fs16\n" + corpo + "}\n"
    )
    (RAIZ / "res/msi/Package/License.rtf").write_text(doc, encoding="ascii")


if __name__ == "__main__":
    main()
