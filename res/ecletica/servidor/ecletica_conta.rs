// Eclética: só conecta quem tem a conta ativa no rustdesk-api.
//
// O hbbs do lejianwen, com MUST_LOGIN, confere apenas a assinatura do JWT de login, que vale
// um ano. Desativar a conta ou apagar o token no rustdesk-api não impedia novas conexões.
// Aqui o token recebido no pedido de conexão é conferido no banco do rustdesk-api: ele precisa
// existir em user_tokens, não estar vencido e pertencer a uma conta com status 1 (ativa).
//
// O resultado fica guardado por 30 segundos para não ler o banco a cada pedido. Se o banco não
// puder ser lido, o pedido segue como antes (com aviso no log): uma falha de leitura nunca
// derruba o acesso de todos.

use hbb_common::{log, tokio::sync::Mutex};
use once_cell::sync::Lazy;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqliteJournalMode},
    ConnectOptions, Connection,
};
use std::{
    collections::HashMap,
    str::FromStr,
    time::{Duration, Instant},
};

const VALIDADE: Duration = Duration::from_secs(30);
const BANCO_PADRAO: &str = "/app/data/rustdeskapi.db";

static GUARDADO: Lazy<Mutex<HashMap<String, (bool, Instant)>>> = Lazy::new(|| Mutex::new(HashMap::new()));

pub const MENSAGEM_BLOQUEIO: &str = "Acesso bloqueado. Fale com o responsável da sua empresa.";

fn caminho_do_banco() -> String {
    std::env::var("ECLETICA_API_DB").unwrap_or_else(|_| BANCO_PADRAO.to_owned())
}

pub async fn ativa(token: &str) -> bool {
    ativa_em(&caminho_do_banco(), token).await
}

pub async fn ativa_em(banco: &str, token: &str) -> bool {
    if let Some((ok, quando)) = GUARDADO.lock().await.get(token) {
        if quando.elapsed() < VALIDADE {
            return *ok;
        }
    }
    let ok = match consultar(banco, token).await {
        Ok(ok) => ok,
        Err(e) => {
            log::warn!("ecletica: banco do rustdesk-api ilegivel ({}), pedido liberado", e);
            return true;
        }
    };
    let mut guardado = GUARDADO.lock().await;
    if guardado.len() > 10_000 {
        guardado.clear();
    }
    guardado.insert(token.to_owned(), (ok, Instant::now()));
    ok
}

async fn consultar(banco: &str, token: &str) -> Result<bool, sqlx::Error> {
    let mut opcoes = SqliteConnectOptions::from_str(&format!("sqlite://{}", banco))?
        .read_only(true)
        // O sqlx 0.6 abre com journal_mode=WAL, que é uma escrita: num banco só leitura em modo
        // "delete" (o do rustdesk-api) isso falhava e todo pedido era liberado.
        .journal_mode(SqliteJournalMode::Delete)
        .busy_timeout(Duration::from_secs(2));
    opcoes.disable_statement_logging();
    let mut con = opcoes.connect().await?;
    let status: Option<(i64,)> = sqlx::query_as(
        "select u.status from user_tokens t join users u on u.id = t.user_id \
         where t.token = ? and t.expired_at > cast(strftime('%s','now') as integer) limit 1",
    )
    .bind(token)
    .fetch_optional(&mut con)
    .await?;
    con.close().await.ok();
    Ok(matches!(status, Some((1,))))
}

#[cfg(test)]
mod testes {
    use super::*;

    async fn banco_de_teste() -> String {
        let caminho = std::env::temp_dir().join(format!("ecletica-conta-{}.db", std::process::id()));
        let caminho = caminho.to_string_lossy().to_string();
        let _ = std::fs::remove_file(&caminho);
        let mut con = SqliteConnectOptions::from_str(&format!("sqlite://{}", caminho))
            .unwrap()
            .create_if_missing(true)
            // Como o banco real do rustdesk-api (bug: abrir só leitura tentava passar para WAL
            // e a conta bloqueada conectava).
            .journal_mode(SqliteJournalMode::Delete)
            .connect()
            .await
            .unwrap();
        for sql in [
            "create table users (id integer primary key, username text, status integer)",
            "create table user_tokens (id integer primary key, user_id integer, token text, expired_at integer)",
            "insert into users values (1, 'ativo@x.com', 1), (2, 'bloqueado@x.com', 2)",
            "insert into user_tokens values (1, 1, 'tok-ativo', 4102444800), (2, 2, 'tok-bloqueado', 4102444800), \
             (3, 1, 'tok-vencido', 1000)",
        ] {
            sqlx::query(sql).execute(&mut con).await.unwrap();
        }
        caminho
    }

    // O hbbs não tem tokio como dependência direta, então o runtime é montado à mão.
    fn rodar<F: std::future::Future>(f: F) -> F::Output {
        hbb_common::tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(f)
    }

    #[test]
    fn so_conta_ativa_com_token_valido_conecta() {
        rodar(async {
            let banco = banco_de_teste().await;
            assert!(ativa_em(&banco, "tok-ativo").await);
            assert!(!ativa_em(&banco, "tok-bloqueado").await, "conta desativada");
            assert!(!ativa_em(&banco, "tok-vencido").await, "token vencido");
            assert!(!ativa_em(&banco, "tok-apagado").await, "token apagado (logout, desaprovado)");
        });
    }

    #[test]
    fn banco_ilegivel_nao_derruba_ninguem() {
        rodar(async {
            assert!(ativa_em("/caminho/que/nao/existe.db", "tok-qualquer").await);
        });
    }
}
