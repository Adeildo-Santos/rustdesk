@AGENTS.md

# Eclética Connect

Fork do RustDesk (AGPL-3.0) vendido pela Eclética Tecnologia Ltda. A branch de
trabalho é `conformidade-agpl`; o build sai pelo workflow "Ecletica - build do
cliente" (`.github/workflows/ecletica-build.yml`, campo `somente`: windows, mac,
android ou todos). Conformidade de licença, assinatura e distribuição:
`res/ecletica/LEIA-ME.md`.

## Testes de regressão (obrigatório)

Todo bug corrigido ganha um teste que falha se ele voltar. Sem teste, a correção
não está pronta.

- **Defeito visível no código** (texto, nome, ícone, link, opção padrão, passo do
  workflow): acrescente uma verificação em `res/ecletica/testes/validar_fonte.py`.
- **Defeito que só aparece no pacote gerado** (instalador, MSI, DMG, arquivos do
  atualizador): acrescente uma verificação em `res/ecletica/testes/validar_pacotes.py`.
- O nome da verificação diz o sintoma do bug, por exemplo
  `"(bug: instalava rustdesk.exe ao lado do antigo)"`.
- Confirme que o teste novo **falha sem a correção** e **passa com ela**. Uma forma é
  rodar o script contra um commit antigo com `git worktree`.

Antes de cada commit, rode (segundos, sem compilar):

```bash
python3 res/ecletica/testes/validar_fonte.py
```

O GitHub Actions roda tudo sozinho:
- `ecletica-testes.yml`: `validar_fonte.py` a cada push nas branches da Eclética e em PRs.
- `ecletica-build.yml`: `validar_fonte.py` antes de compilar (se falhar, nem compila)
  e `validar_pacotes.py` depois, abrindo o instalador e os DMGs publicados.

Para validar um Release na mão (precisa de 7-Zip, msitools e `pip install pefile`;
os DMGs só abrem no macOS):

```bash
python3 res/ecletica/testes/validar_pacotes.py --versao 1.5.26 --pasta ./pacotes --repo Adeildo-Santos/rustdesk
```

## Regras que os testes protegem

- **Versão:** ao subir a versão, mude todos os lugares juntos: `Cargo.toml`,
  `Cargo.lock` (pacotes `rustdesk` e `rustdesk-portable-packer`),
  `libs/portable/Cargo.toml`, `flutter/pubspec.yaml` e `VERSION` em
  `flutter-build.yml`. Nunca reaproveite um número já publicado: a tag da versão
  precisa apontar para o commit compilado (código-fonte correspondente da AGPL).
- **Nunca mudar:**
  - `DATA_DIR_NAME` (`libs/hbb_common/src/config.rs`): guarda a ID e a senha das máquinas.
  - `UPGRADE_CODE_SEED` (`res/msi/preprocess.py`): sem ele, o MSI instala duplicado em vez de atualizar.
- **Licença:** os avisos legais ficam no "Sobre", no `AVISOS-LEGAIS.txt` e no
  `LICENCE.txt` instalados. Ao mudar `AVISOS-LEGAIS.txt` ou `LICENCE`, rode
  `python3 res/ecletica/gerar_licenca_rtf.py` para atualizar a tela de licença do MSI.
- **Marca:** "RustDesk" só aparece como crédito ("baseado no RustDesk"), nunca como
  nome do produto, ícone, link de suporte ou nome de arquivo instalado.
- **Releases:** mantenha só a versão atual (a do `rustdesk-versao.json` e a do
  instalador distribuído) e apague as anteriores.
