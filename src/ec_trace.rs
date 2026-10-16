//! Ecletica: rastro de diagnostico em arquivo.
//!
//! O binario da interface e de janela (windows_subsystem = "windows"): nao tem
//! console, entao quando a tela nao desenha nao sobra pista nenhuma. Este modulo
//! grava um arquivo de rastro na pasta de log do cliente. Nao depende de nenhuma
//! biblioteca nova e nao muda o funcionamento do programa: apenas escreve linhas.

use std::io::Write;

/// Caminho do arquivo de rastro (mesma pasta de log que o cliente usa).
pub fn caminho() -> std::path::PathBuf {
    let mut p = hbb_common::config::Config::log_path();
    if !p.exists() {
        let _ = std::fs::create_dir_all(&p);
    }
    p.push("ecletica-rastro.txt");
    p
}

/// Escreve uma linha no rastro. Nunca entra em panico e nunca falha para fora.
pub fn linha(msg: &str) {
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(caminho())
    {
        let _ = writeln!(f, "{} {}", t, msg);
    }
}

/// Grava qualquer panico (inclusive em thread da interface) no rastro.
pub fn instala_gancho_de_panico() {
    let anterior = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        linha(&format!("PANICO: {}", info));
        anterior(info);
    }));
}

/// Ecletica: garante UM cliente por maquina.
///
/// Duas copias abertas ao mesmo tempo registram a mesma maquina com identidades
/// diferentes e a conexao passa a falhar no servidor (chave trocada para o mesmo
/// ID). A primeira copia segura uma porta local; a segunda nao consegue e fecha.
///
/// Devolve `true` quando esta copia pode seguir (ela e a dona da porta).
pub fn segura_copia_unica() -> bool {
    let porta: u16 = 42548;
    match std::net::TcpListener::bind(("127.0.0.1", porta)) {
        Ok(ouvinte) => {
            // Deixa o ouvinte vivo enquanto o programa estiver rodando: e' de proposito
            // que nao o devolvemos ao sistema (equivale a varavel global, sem precisar
            // de `static`, que o compilador antigo do 32 bits nao aceita aqui).
            std::mem::forget(ouvinte);
            true
        }
        Err(e) => {
            if std::net::TcpStream::connect(("127.0.0.1", porta)).is_ok() {
                false
            } else {
                // Porta ocupada por outro programa: nao impedimos o cliente de abrir.
                linha(&format!(
                    "aviso: porta {} ocupada por outro programa ({}) - o cliente segue",
                    porta, e
                ));
                true
            }
        }
    }
}
