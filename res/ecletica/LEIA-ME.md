# Eclética Connect: conformidade e distribuição

## O que o build já faz

- Instala `LICENCE.txt` (AGPL-3.0) e `AVISOS-LEGAIS.txt` junto com o programa (32 e 64 bits).
- A tela de licença do MSI mostra os avisos legais e a AGPL (`res/msi/Package/License.rtf`).
  Se `AVISOS-LEGAIS.txt` ou `LICENCE` mudarem, gere o RTF de novo:
  `python3 res/ecletica/gerar_licenca_rtf.py`
- Cada versão publicada ganha uma tag com o número da versão apontando para o commit
  compilado, e a página do Release tem o código-fonte dessa versão. Esse é o link
  "Código-fonte" das telas "Sobre".
- Gera um instalador único, `Ecletica-Connect-Setup-Windows.exe` (job `instalador-unico`),
  que mostra a licença e instala o MSI no Windows 64 bits ou o cliente Sciter no 32 bits.

## Assinatura digital (Authenticode)

Sem certificado, todos os arquivos saem sem assinatura (o build avisa e continua).
Para assinar, cadastre em Settings → Secrets and variables → Actions:

- `WINDOWS_PFX_BASE64`: o certificado `.pfx` em base64 (`base64 -i certificado.pfx`)
- `WINDOWS_PFX_PASSWORD`: a senha do `.pfx`

Certificados emitidos depois de junho de 2023 ficam em token ou HSM na nuvem e não
saem como `.pfx`. Nesse caso, troque o corpo de `res/ecletica/assinar.ps1` pela
ferramenta do emissor (por exemplo, SSL.com eSigner, DigiCert KeyLocker ou Azure
Artifact Signing). As chamadas no workflow continuam as mesmas.

## Fora do código (fazer antes de vender)

- Publicar a Política de Privacidade (LGPD) e os Termos de Uso em www.ecletica.com.br.
  Os links "Privacy Statement" apontam para o site; troque pelo endereço da política
  quando ela existir.
- Contrato com o cliente: cobrar pelo serviço (servidores, suporte, assinatura), sem
  cláusulas que proíbam redistribuir ou estudar o software.
- Se o servidor (hbbs/hbbr ou rustdesk-api-server-pro) em app.ecletico.ai tiver
  modificações, publicar o código-fonte dele também (AGPL, seção 13).
- Confirmar por e-mail com o RustDesk que o uso do nome "baseado no RustDesk" está ok.
