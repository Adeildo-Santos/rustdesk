#!/usr/bin/env python3
"""Aplica a mudança da Eclética no código do lejianwen/rustdesk-server (hbbs).

Uso: python3 aplicar.py <pasta do rustdesk-server>

Copia ecletica_conta.rs para src/, registra o módulo em src/lib.rs e, no pedido de conexão
(handle_punch_hole_request), recusa o token de uma conta que não está ativa no rustdesk-api.
Cada âncora precisa aparecer uma única vez: se o código de origem mudar, o script para em vez
de aplicar no lugar errado.
"""
import os
import shutil
import sys

AQUI = os.path.dirname(os.path.abspath(__file__))

ANCORA_LOGIN = """        // if secret is not empty check token by jwt
        if MUST_LOGIN.load(Ordering::SeqCst) {"""

VERIFICACAO = """        // Eclética: além da assinatura do JWT, a conta precisa estar ativa no rustdesk-api
        // (teste encerrado, acesso retirado, plano vencido ou cancelado bloqueiam aqui).
        if MUST_LOGIN.load(Ordering::SeqCst)
            && !ph.token.is_empty()
            && !crate::ecletica_conta::ativa(&ph.token).await
        {
            let mut msg_out = RendezvousMessage::new();
            msg_out.set_punch_hole_response(PunchHoleResponse {
                other_failure: String::from(crate::ecletica_conta::MENSAGEM_BLOQUEIO),
                ..Default::default()
            });
            return Ok((msg_out, None));
        }
"""


def trocar_uma_vez(texto, antes, depois, arquivo):
    n = texto.count(antes)
    if n != 1:
        sys.exit("aplicar.py: âncora encontrada %d vezes em %s (esperado 1). Código de origem mudou?" % (n, arquivo))
    return texto.replace(antes, depois, 1)


def main():
    raiz = sys.argv[1]
    src = os.path.join(raiz, "src")
    shutil.copy(os.path.join(AQUI, "ecletica_conta.rs"), os.path.join(src, "ecletica_conta.rs"))

    lib = os.path.join(src, "lib.rs")
    texto = open(lib, encoding="utf-8").read()
    if "mod ecletica_conta;" not in texto:
        texto = trocar_uma_vez(texto, "mod rendezvous_server;", "mod ecletica_conta;\nmod rendezvous_server;", lib)
        open(lib, "w", encoding="utf-8").write(texto)

    rs = os.path.join(src, "rendezvous_server.rs")
    texto = open(rs, encoding="utf-8").read()
    if "crate::ecletica_conta::ativa" not in texto:
        texto = trocar_uma_vez(texto, ANCORA_LOGIN, VERIFICACAO + ANCORA_LOGIN, rs)
        open(rs, "w", encoding="utf-8").write(texto)
    print("Mudança da Eclética aplicada em", raiz)


if __name__ == "__main__":
    main()
