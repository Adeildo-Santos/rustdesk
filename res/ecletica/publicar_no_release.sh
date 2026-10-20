#!/usr/bin/env bash
# Envia arquivos para um Release do GitHub (cria o Release se ainda nao existir).
# Uso: publicar_no_release.sh <tag> <arquivo>...
# Substitui o softprops/action-gh-release nos Releases reaproveitados (ecletica): a
# atualizacao que a action faz no Release passou a ser recusada pelo GitHub
# ("Resource not accessible by integration"), enquanto o envio pelo gh funciona.
set -euo pipefail
T="$1"; shift
R="${GITHUB_REPOSITORY}"
if ! gh release view "$T" -R "$R" > /dev/null 2>&1; then
  gh release create "$T" -R "$R" --prerelease --target "${GITHUB_SHA}" --title "$T" --notes "" || true
fi
for tentativa in 1 2 3; do
  if gh release upload "$T" "$@" -R "$R" --clobber; then
    echo "enviado para $T: $*"
    exit 0
  fi
  sleep 15
done
echo "falha ao enviar para $T" >&2
exit 1
