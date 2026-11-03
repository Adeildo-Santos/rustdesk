# Servidor de conexão (hbbs) da Eclética

O servidor da Eclética usa a imagem `lejianwen/rustdesk-server-s6` (hbbs + hbbr + rustdesk-api).
Com `MUST_LOGIN=Y`, o hbbs original confere só a **assinatura** do JWT de login, que vale um
ano. Desativar a conta ou apagar o token no rustdesk-api não impedia novas conexões.

## A mudança

`ecletica_conta.rs` faz o hbbs, antes de cada pedido de conexão, conferir no banco do
rustdesk-api (`/app/data/rustdeskapi.db`, só leitura, no mesmo container) três coisas:

- o token existe em `user_tokens`;
- o token não está vencido;
- a conta está ativa (`status = 1`).

Se não, o pedido é recusado com "Acesso bloqueado. Fale com o responsável da sua empresa."

Isso cobre o teste grátis encerrado, o acesso retirado pelo responsável, o funcionário
removido e o plano vencido ou cancelado, para qualquer versão do programa.

- **Tempo de efeito:** o resultado fica guardado 30 segundos, então o bloqueio vale em até 30 s.
- **Sessões abertas:** uma sessão já aberta continua até ser fechada; a próxima é recusada.
- **Falha de leitura:** se o banco não puder ser lido, o pedido segue como antes, com aviso no
  log. Uma falha de leitura nunca derruba o acesso de todos.

## Como é montado

- **`aplicar.py`:** aplica a mudança sobre o código do `lejianwen/rustdesk-server` no commit
  fixado em `.github/workflows/ecletica-hbbs.yml`. Ele confere cada âncora e para se o código
  de origem mudou.
- **O workflow:** testa (`cargo test --lib ecletica_conta`), compila para Linux musl e publica o
  binário e o `sha256` no Release `servidor-hbbs`.
- **No servidor:** o binário é baixado, conferido pelo `sha256` e montado por cima do
  `/usr/bin/hbbs` da imagem. Em seguida o container é reiniciado.
- **Código-fonte (AGPL):** o código de origem fixado mais esta pasta.

## Volta

Tire do `docker-compose.yml` o volume que monta o binário em `/usr/bin/hbbs` e reinicie o
container. O hbbs original da imagem volta a valer.
