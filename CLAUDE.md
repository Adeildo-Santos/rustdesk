@AGENTS.md

# Eclética Connect

Acesso remoto vendido pela Eclética Tecnologia Ltda. É um fork do RustDesk
(AGPL-3.0) com marca, servidores e instaladores próprios. Pode ser vendido; a AGPL
exige código-fonte aberto, avisos legais e nenhuma restrição extra ao cliente.
Conformidade, assinatura e distribuição: `res/ecletica/LEIA-ME.md`.

## Repositório e branches

- Fork: `github.com/Adeildo-Santos/rustdesk`. Upstream: `rustdesk/rustdesk`.
- **Branch de trabalho: `conformidade-agpl`.** Ela gera Windows (32, 64 e ARM64) e Mac
  (Apple Silicon e Intel) com o mesmo código e a mesma versão.
- Branches antigas (`ecletica`, `ecletica-32`, `ecletica-connect-x86`, `ecletica-limpo`,
  `ecletica-x86-msi`, `ecletica-mac-1.5.0`) são histórico; não gere build a partir delas.
- `libs/hbb_common` está **copiado no repositório** (não é submódulo) e tem mudanças
  da Eclética (servidor, chave, pasta de dados).

## Onde fica cada coisa da Eclética

| Assunto | Arquivo |
|---|---|
| Servidor ID/relay, chave, API (`app.ecletico.ai`) | `libs/hbb_common/src/config.rs` (`RENDEZVOUS_SERVERS`, `RS_PUB_KEY`, `Config2::load`) |
| Nome do app ("Ecletica Connect") e pasta de dados ("Ecletica Acesso Remoto") | `libs/hbb_common/src/config.rs` (`APP_NAME`, `DATA_DIR_NAME`) |
| Atualização automática (lê `app.ecletico.ai/rustdesk-versao.json`) | `src/common.rs` (`ECLETICA_VERSION_JSON`), `src/updater.rs` |
| Opções fixas: sem "Desenvolvido por RustDesk", "Somente visualizar" desligado | `src/common.rs` (`apply_ecletica_settings`), `src/client.rs` (`initialize`) |
| Instalação/desinstalação Windows sem MSI (32 bits) | `src/platform/windows.rs` (`install_me`, `get_uninstall`) |
| Tela "Sobre" com avisos legais | `flutter/lib/desktop/pages/desktop_setting_page.dart`, `src/ui/index.tis` (32 bits), `flutter/lib/mobile/pages/settings_page.dart` |
| Interface 32 bits (Sciter) e logo | `src/ui/index.tis`, `src/ui/index.css`, ícone embutido em `src/ui.rs` (`get_icon`) |
| Logos e ícones | `flutter/assets/`, `res/*.png`, `res/*.ico`, `flutter/macos/Runner/AppIcon.icns`, `flutter/windows/runner/resources/app_icon.ico` |
| Metadados dos executáveis | `Cargo.toml` e `libs/portable/Cargo.toml` (`[package.metadata.winres]`), `flutter/windows/runner/Runner.rc` |
| Mac: nome, identificador `br.com.ecletica.connect`, copyright | `flutter/macos/Runner/Info.plist`, `Configs/AppInfo.xcconfig`, `Runner.xcodeproj/project.pbxproj` |
| Android: `applicationId br.com.ecletica.connect` | `flutter/android/app/build.gradle` |
| MSI 64 bits (licença, links, serviço, UpgradeCode) | `res/msi/preprocess.py`, `res/msi/Package/` |
| Instalador único 32/64 bits (silencioso) | `res/ecletica/instalador.nsi` |
| Avisos legais e licença | `AVISOS-LEGAIS.txt`, `LICENCE`, `res/msi/Package/License.rtf` (gerado por `res/ecletica/gerar_licenca_rtf.py`) |
| Assinatura Authenticode (opcional, segredos `WINDOWS_PFX_*`) | `res/ecletica/assinar.ps1` |
| Build e publicação | `.github/workflows/ecletica-build.yml` → `flutter-build.yml` |

## Gerar uma versão

1. Suba a versão em **todos** os lugares juntos (o teste confere): `Cargo.toml`,
   `Cargo.lock` (pacotes `rustdesk` e `rustdesk-portable-packer`),
   `libs/portable/Cargo.toml`, `flutter/pubspec.yaml` (`x.y.z+N`, N sobe 1) e
   `VERSION` em `flutter-build.yml`. Nunca reaproveite um número já publicado.
2. `python3 res/ecletica/testes/validar_fonte.py` precisa passar.
3. Commit, push em `conformidade-agpl` e dispare com o campo `tag` **vazio**:
   ```bash
   gh workflow run ecletica-build.yml -R Adeildo-Santos/rustdesk --ref conformidade-agpl -f somente=todos
   ```
   `somente`: `todos` (Windows + Mac), `windows`, `mac` ou `android`. Leva ~1 hora.
4. O workflow roda `validar_fonte.py` antes, compila, publica e roda
   `validar_pacotes.py` nos arquivos publicados.

O build publica em dois Releases:
- **`ecletica`**: instaladores para distribuir + `VERSION.txt`. Links fixos, por exemplo
  `releases/download/ecletica/Ecletica-Connect-Setup-Windows.exe`.
- **`<versão>`** (ex.: `1.5.27`): arquivos `rustdesk-<versão>-<arch>.<ext>` que o
  atualizador baixa. A tag aponta para o commit compilado (código-fonte da AGPL).

## Atualização automática dos clientes

- O servidor `app.ecletico.ai` (Ubuntu, acesso por `ssh ecletica`, usuário sem sudo)
  roda como root, a cada 15 minutos, uma rotina que lê o `VERSION.txt` do Release
  **`ecletica`**, confere os arquivos do Release `<versão>` e publica
  `/www/wwwroot/app.ecletico.ai/rustdesk-versao.json`. Log: `/var/log/rustdesk_versao.log`.
- **Nunca apague o Release `ecletica`.** Sem ele, a rotina para ("NAO PUBLICA") e os
  clientes deixam de se atualizar.
- Os clientes consultam o JSON a cada 10 minutos. O 32 bits baixa
  `rustdesk-<versão>-x86-sciter.exe`; o 64 bits, o `.msi`; o Mac, o `.dmg`.
- Limpeza: depois que o JSON mostrar a versão nova, apague Releases e tags de versões
  antigas. Fica só `ecletica`, `<versão atual>` e, se existir, `ecletica-<versão atual>`.

## Testes de regressão (obrigatório)

Todo bug corrigido ganha um teste que falha se ele voltar. Sem teste, a correção
não está pronta.

- **Defeito visível no código** (texto, nome, ícone, link, opção padrão, passo do
  workflow): acrescente uma verificação em `res/ecletica/testes/validar_fonte.py`.
- **Defeito que só aparece no pacote gerado** (instalador, MSI, DMG, arquivos do
  atualizador): acrescente uma verificação em `res/ecletica/testes/validar_pacotes.py`.
- O nome da verificação diz o sintoma do bug, por exemplo
  `"(bug: instalava rustdesk.exe ao lado do antigo)"`.
- Confirme que o teste novo **falha sem a correção** e **passa com ela**: copie o
  script para um `git worktree` do commit anterior e rode lá.

Antes de cada commit (segundos, sem compilar):

```bash
python3 res/ecletica/testes/validar_fonte.py
```

O GitHub Actions roda tudo sozinho:
- `ecletica-testes.yml`: `validar_fonte.py` a cada push nas branches da Eclética e em PRs.
- `ecletica-build.yml`: `validar_fonte.py` antes de compilar e `validar_pacotes.py` depois.

Para validar um Release na mão (precisa de 7-Zip, msitools e `pip install pefile`;
os DMGs só abrem no macOS):

```bash
python3 res/ecletica/testes/validar_pacotes.py --versao 1.5.27 --pasta ./pacotes --repo Adeildo-Santos/rustdesk
```

## Regras que não podem quebrar

- **Nunca mudar:**
  - `DATA_DIR_NAME = "Ecletica Acesso Remoto"`: é a pasta que guarda a ID e a senha
    das máquinas. Mudar dá ID novo a todos os clientes.
  - `UPGRADE_CODE_SEED = "RustDesk.exe"` (`res/msi/preprocess.py`): sem ele, o MSI
    instala duplicado em vez de atualizar.
  - `ORG = "com.carriez"` e os nomes `com.carriez.*` do Mac: guardam a configuração
    dos Macs já instalados.
- **Nome com espaço:** o app se chama "Ecletica Connect". Todo comando de shell, `sc`,
  `taskkill` ou `reg` que usa o nome ou a chave de registro precisa de **aspas**.
  Sem aspas o Windows recusa o comando (bug da desinstalação do 32 bits).
- **32 bits:** a instalação copia a pasta sem renomear, então o executável já sai do
  build como `Ecletica Connect.exe`.
- **Licença:** avisos no "Sobre", no `AVISOS-LEGAIS.txt` e no `LICENCE.txt` instalados.
  Ao mudar um dos dois arquivos, rode `python3 res/ecletica/gerar_licenca_rtf.py`.
  Sciter (32 bits) exige o crédito "uses Sciter Engine" no "Sobre".
- **Marca:** "RustDesk" só aparece como crédito ("baseado no RustDesk"), nunca como
  nome do produto, ícone, link de suporte ou arquivo instalado.
- **Instalador único:** silencioso (só o UAC do Windows). Não volte a pôr telas.

## Mac

- Sem conta Apple Developer: o app sai com assinatura **ad-hoc** (reassinado no fim do
  build) e sem notarização. Na primeira abertura, botão direito → Abrir.
- Cada build ad-hoc tem uma identidade nova: as permissões (Gravação de Tela,
  Acessibilidade, Monitoramento de Entrada) precisam ser dadas de novo após atualizar.
- "Mac conecta mas não clica nem digita" já foi "Somente visualizar" gravado nas
  conexões (`~/Library/Preferences/com.carriez.Ecletica-Acesso-Remoto/peers/*.toml`,
  `view_only`). Desde a 1.5.26 o código desliga isso em toda sessão.
- Logs do Mac: `~/Library/Logs/Ecletica Acesso Remoto/`.

## Pendências fora do código

- Certificado de assinatura de código Windows (segredos `WINDOWS_PFX_BASE64` e
  `WINDOWS_PFX_PASSWORD`) e conta Apple Developer para o Mac.
- Política de Privacidade (LGPD) e Termos de Uso publicados em www.ecletica.com.br.
- Contrato cobrando pelo serviço (servidores e suporte), sem proibir redistribuição.
- Se o servidor (hbbs/hbbr ou API) tiver modificações, publicar o código dele (AGPL §13).
- O build Windows ARM64 às vezes falha baixando dependências do Google
  (aomedia.googlesource.com); basta rodar de novo.
