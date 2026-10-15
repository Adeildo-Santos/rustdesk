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
