# Assina (Authenticode) os .exe, .dll e .msi ainda sem assinatura dentro das pastas dadas.
# Sem os segredos WINDOWS_PFX_BASE64 / WINDOWS_PFX_PASSWORD no repositorio, so avisa e sai
# com sucesso: o build continua, mas os arquivos saem sem assinatura.
param([Parameter(Mandatory = $true)][string[]]$Pastas)

if (-not $env:WINDOWS_PFX_BASE64) {
    Write-Host "::warning::Sem certificado (segredo WINDOWS_PFX_BASE64): arquivos saem SEM assinatura digital."
    exit 0
}

$signtool = Get-ChildItem "${env:ProgramFiles(x86)}\Windows Kits\10\bin\*\x64\signtool.exe" |
    Sort-Object FullName -Descending | Select-Object -First 1
if (-not $signtool) { Write-Error "signtool.exe nao encontrado"; exit 1 }

$pfx = Join-Path $env:RUNNER_TEMP "ecletica-assinatura.pfx"
[IO.File]::WriteAllBytes($pfx, [Convert]::FromBase64String($env:WINDOWS_PFX_BASE64))
try {
    foreach ($pasta in $Pastas) {
        Get-ChildItem -Path $pasta -Recurse -File -Include *.exe, *.dll, *.msi | ForEach-Object {
            if ((Get-AuthenticodeSignature $_.FullName).Status -eq 'NotSigned') {
                & $signtool.FullName sign /f $pfx /p $env:WINDOWS_PFX_PASSWORD /fd sha256 `
                    /tr http://timestamp.digicert.com /td sha256 /d "Eclética Connect" $_.FullName
                if ($LASTEXITCODE -ne 0) { throw "falha ao assinar $($_.FullName)" }
            }
        }
    }
} finally {
    Remove-Item -Force $pfx -ErrorAction SilentlyContinue
}
