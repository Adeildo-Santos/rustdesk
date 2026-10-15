use crate::log;
use directories_next::ProjectDirs;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};
use sodiumoxide::crypto::sign;
use std::{
    collections::HashMap,
    fs,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
    time::SystemTime,
};

pub const APP_NAME: &str = "Ecletica Connect";
// Ecletica: nome da pasta de dados, dos arquivos de configuracao e do log.
// Continua o antigo ("RustDesk") de proposito: e onde mora o ID e a configuracao
// das maquinas ja instaladas. Trocar esse nome daria um ID novo para cada maquina
// e perderia a configuracao. O nome mostrado ao usuario e APP_NAME.
pub const DATA_NAME: &str = "RustDesk";
pub const RENDEZVOUS_TIMEOUT: u64 = 12_000;
pub const CONNECT_TIMEOUT: u64 = 18_000;
pub const COMPRESS_LEVEL: i32 = 3;
const SERIAL: i32 = 1;
// 128x128
#[cfg(target_os = "macos")] // 128x128 on 160x160 canvas, then shrink to 128, mac looks better with padding
pub const ICON: &str = "data:image/png;base64,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";
#[cfg(windows)] // windows, 32x32, bigger very ugly after shrink
pub const ICON: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAYAAABzenr0AAADYklEQVR42u1WPWwcRRT+3pvd8y0xCQVSDFWQCBQuaFxEIhI/ogBFWEqxJ3Q5WQGF0IBEZCkl6wVRBAubCiQXOZzjLuIWuYiEGwpjGloiYToXtEEpkNAl2duZL8XdXu7s83E2yFD4SW93387fN/O++WaAIzuy/9jMYQxCUgDo5uYmd5Z543YSRZFOT0/LQQCIiAXAZrNpSqWS3Xf7KIr0n6xAvV4/Xa/XT3cnMjDpsWe0trb2XJqmz6RpOqRN3mc2EBtDdU4za9uvGONdIlGemyv/MC4AIQkAcuNG7UtRMwcy6FvW/hwP/MvjPOHOOYCEqnEAF0X4yfb29r04jrnn0jabTRURrq7Wvpiaeuo9z5jA8zx4no/Ou+PGmIHv/tjvurPWkYTvewrghcnJyeLCwgJJjk7B+vr6xJ07dz81Ro9ZawFwbC5kGY0qHKAWsO94nn9XFVcqlcq347MnDP+Vbbq6WvuoWq1O5btpTO4dnPmL8/PHXpr97F0VYGNjw+tL664J7YGEAgjPvrn0YtsVzzhmPoRU9GNyvacCcKQAik49BPdSiTxxN2dO/XZpZWWlFYZNkyS7NUCGKI4ijt3M659fS13xKikQCAABe7XZdQGIbvkj3lMAm93PjCl6Bunvx4P2+Z9uzf+CaEEQx65/OB1MedMgjt2Zc0sX6J+8CphO5wJQHAQcdCKH1vVO7LIHUDEeYKHG3DYwLQBAvLeCAACSZIsA8PjkxB+pu385U+tERjHfdgVDCXUGFBEjaavFt9tWnn3Mxwc/f//+d526V6SXt1E2jCh/q9U74pfPL517bfba0492EmXsA+dgWn/ryevXa18tL1ef2JXSccFHUaRxHLuvv7n5hiFLmeVxRwfhbvQOhOayCxCk5/uFWTp3m5QLFy+Wt/Zi/VAA+TFZrdY+LAbBsnMOJNAn9724y/2e3ufWarXSIAgKJKEir1Yqb/0YhqEmSTIShBdFkZZKJVurNZ/PbLqcpg+QZRlEFCQHQOQHjYgMAHCOaRAEhczavxRYLAT+r91yJzI6/UJSRISNRuOk7/un/my17IROCNAeUt1H//8sg1E1mXXZx75fOGHUXC6Xw61Dv9c1GslMTuCdF46xSUhSkiTZ1y4Iw5Dde4DrJ/Khr8A+TrgjO7L/nz0ErqCHOvCX3/IAAAAASUVORK5CYII=";
#[cfg(target_os = "linux")] // 128x128 no padding
pub const ICON: &str = "data:image/png;base64,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";
#[cfg(target_os = "macos")]
pub const ORG: &str = "ai.ecletico";

type Size = (i32, i32, i32, i32);

lazy_static::lazy_static! {
    static ref CONFIG: Arc<RwLock<Config>> = Arc::new(RwLock::new(Config::load()));
    static ref CONFIG2: Arc<RwLock<Config2>> = Arc::new(RwLock::new(Config2::load()));
    pub static ref ONLINE: Arc<Mutex<HashMap<String, i64>>> = Default::default();
}
#[cfg(any(target_os = "android", target_os = "ios"))]
lazy_static::lazy_static! {
    pub static ref APP_DIR: Arc<RwLock<String>> = Default::default();
}
const CHARS: &'static [char] = &[
    '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k',
    'm', 'n', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

pub const RENDEZVOUS_SERVERS: &'static [&'static str] = &[
    "rs-ny.rustdesk.com",
    "rs-sg.rustdesk.com",
    "rs-cn.rustdesk.com",
];
pub const RENDEZVOUS_PORT: i32 = 21116;
pub const RELAY_PORT: i32 = 21117;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NetworkType {
    Direct,
    ProxySocks,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default)]
    id: String,
    #[serde(default)]
    password: String,
    #[serde(default)]
    salt: String,
    #[serde(default)]
    key_pair: (Vec<u8>, Vec<u8>), // sk, pk
    #[serde(default)]
    key_confirmed: bool,
    #[serde(default)]
    keys_confirmed: HashMap<String, bool>,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
pub struct Socks5Server {
    #[serde(default)]
    pub proxy: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
}

// more variable configs
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Config2 {
    #[serde(default)]
    remote_id: String, // latest used one
    #[serde(default)]
    size: Size,
    #[serde(default)]
    rendezvous_server: String,
    #[serde(default)]
    nat_type: i32,
    #[serde(default)]
    serial: i32,

    #[serde(default)]
    socks: Option<Socks5Server>,

    // the other scalar value must before this
    #[serde(default)]
    pub options: HashMap<String, String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct PeerConfig {
    #[serde(default)]
    pub password: Vec<u8>,
    #[serde(default)]
    pub size: Size,
    #[serde(default)]
    pub size_ft: Size,
    #[serde(default)]
    pub size_pf: Size,
    #[serde(default)]
    pub view_style: String, // original (default), scale
    #[serde(default)]
    pub image_quality: String,
    #[serde(default)]
    pub custom_image_quality: Vec<i32>,
    #[serde(default)]
    pub show_remote_cursor: bool,
    #[serde(default)]
    pub lock_after_session_end: bool,
    #[serde(default)]
    pub privacy_mode: bool,
    #[serde(default)]
    pub port_forwards: Vec<(i32, String, i32)>,
    #[serde(default)]
    pub direct_failures: i32,
    #[serde(default)]
    pub disable_audio: bool,
    #[serde(default)]
    pub disable_clipboard: bool,

    // the other scalar value must before this
    #[serde(default)]
    pub options: HashMap<String, String>,
    #[serde(default)]
    pub info: PeerInfoSerde,
}

#[derive(Debug, PartialEq, Default, Serialize, Deserialize, Clone)]
pub struct PeerInfoSerde {
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub hostname: String,
    #[serde(default)]
    pub platform: String,
}

fn patch(path: PathBuf) -> PathBuf {
    if let Some(_tmp) = path.to_str() {
        #[cfg(windows)]
        return _tmp
            .replace(
                "system32\\config\\systemprofile",
                "ServiceProfiles\\LocalService",
            )
            .into();
        #[cfg(target_os = "macos")]
        return _tmp.replace("Application Support", "Preferences").into();
        #[cfg(target_os = "linux")]
        {
            if _tmp == "/root" {
                if let Ok(output) = std::process::Command::new("whoami").output() {
                    let user = String::from_utf8_lossy(&output.stdout)
                        .to_string()
                        .trim()
                        .to_owned();
                    if user != "root" {
                        return format!("/home/{}", user).into();
                    }
                }
            }
        }
    }
    path
}

impl Config2 {
    fn load() -> Config2 {
        Config::load_::<Config2>("2")
    }

    fn store(&self) {
        Config::store_(self, "2");
    }
}

impl Config {
    fn load_<T: serde::Serialize + serde::de::DeserializeOwned + Default + std::fmt::Debug>(
        suffix: &str,
    ) -> T {
        let file = Self::file_(suffix);
        log::debug!("Configuration path: {}", file.display());
        let cfg = match confy::load_path(&file) {
            Ok(config) => config,
            Err(err) => {
                log::error!("Failed to load config: {}", err);
                T::default()
            }
        };
        if suffix.is_empty() {
            log::debug!("{:?}", cfg);
        }
        cfg
    }

    fn store_<T: serde::Serialize>(config: &T, suffix: &str) {
        let file = Self::file_(suffix);
        if let Err(err) = confy::store_path(file, config) {
            log::error!("Failed to store config: {}", err);
        }
    }

    fn load() -> Config {
        Config::load_::<Config>("")
    }

    fn store(&self) {
        Config::store_(self, "");
    }

    pub fn file() -> PathBuf {
        Self::file_("")
    }

    pub fn import(from: &str) {
        log::info!("import {}", from);
        // load first to create path
        Self::load();
        crate::allow_err!(std::fs::copy(from, Self::file()));
        crate::allow_err!(std::fs::copy(
            from.replace(".toml", "2.toml"),
            Self::file_("2")
        ));
    }

    pub fn save_tmp() -> String {
        let _lock = CONFIG.read().unwrap(); // do not use let _, which will be dropped immediately
        let path = Self::file_("2").to_str().unwrap_or("").to_owned();
        let path2 = format!("{}_tmp", path);
        crate::allow_err!(std::fs::copy(&path, &path2));
        let path = Self::file().to_str().unwrap_or("").to_owned();
        let path2 = format!("{}_tmp", path);
        crate::allow_err!(std::fs::copy(&path, &path2));
        path2
    }

    fn file_(suffix: &str) -> PathBuf {
        let name = format!("{}{}", DATA_NAME, suffix);
        Self::path(name).with_extension("toml")
    }

    pub fn get_home() -> PathBuf {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        return Self::path("");
        if let Some(path) = dirs_next::home_dir() {
            patch(path)
        } else if let Ok(path) = std::env::current_dir() {
            path
        } else {
            std::env::temp_dir()
        }
    }

    fn path<P: AsRef<Path>>(p: P) -> PathBuf {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            let mut path: PathBuf = APP_DIR.read().unwrap().clone().into();
            path.push(p);
            return path;
        }
        #[cfg(not(target_os = "macos"))]
        let org = "";
        #[cfg(target_os = "macos")]
        let org = ORG;
        // /var/root for root
        if let Some(project) = ProjectDirs::from("", org, DATA_NAME) {
            let mut path = patch(project.config_dir().to_path_buf());
            path.push(p);
            return path;
        }
        return "".into();
    }

    #[allow(unreachable_code)]
    pub fn log_path() -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            if let Some(path) = dirs_next::home_dir().as_mut() {
                path.push(format!("Library/Logs/{}", APP_NAME));
                return path.clone();
            }
        }
        #[cfg(target_os = "linux")]
        {
            let mut path = Self::get_home();
            path.push(format!(".local/share/logs/{}", APP_NAME));
            std::fs::create_dir_all(&path).ok();
            return path;
        }
        if let Some(path) = Self::path("").parent() {
            let mut path: PathBuf = path.into();
            path.push("log");
            return path;
        }
        "".into()
    }

    pub fn ipc_path(postfix: &str) -> String {
        #[cfg(windows)]
        {
            // \\ServerName\pipe\PipeName
            // where ServerName is either the name of a remote computer or a period, to specify the local computer.
            // https://docs.microsoft.com/en-us/windows/win32/ipc/pipe-names
            format!("\\\\.\\pipe\\{}\\query{}", APP_NAME, postfix)
        }
        #[cfg(not(windows))]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut path: PathBuf = format!("/tmp/{}", APP_NAME).into();
            fs::create_dir(&path).ok();
            fs::set_permissions(&path, fs::Permissions::from_mode(0o0777)).ok();
            path.push(format!("ipc{}", postfix));
            path.to_str().unwrap_or("").to_owned()
        }
    }

    pub fn icon_path() -> PathBuf {
        let mut path = Self::path("icons");
        if fs::create_dir_all(&path).is_err() {
            path = std::env::temp_dir();
        }
        path
    }

    #[inline]
    pub fn get_any_listen_addr() -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), 0)
    }

    pub fn get_rendezvous_server() -> String {
        let mut rendezvous_server = Self::get_option("custom-rendezvous-server");
        if rendezvous_server.is_empty() {
            rendezvous_server = CONFIG2.write().unwrap().rendezvous_server.clone();
        }
        if rendezvous_server.is_empty() {
            rendezvous_server = Self::get_rendezvous_servers()
                .drain(..)
                .next()
                .unwrap_or("".to_owned());
        }
        if !rendezvous_server.contains(":") {
            rendezvous_server = format!("{}:{}", rendezvous_server, RENDEZVOUS_PORT);
        }
        rendezvous_server
    }

    pub fn get_rendezvous_servers() -> Vec<String> {
        let s = Self::get_option("custom-rendezvous-server");
        if !s.is_empty() {
            return vec![s];
        }
        let serial_obsolute = CONFIG2.read().unwrap().serial > SERIAL;
        if serial_obsolute {
            let ss: Vec<String> = Self::get_option("rendezvous-servers")
                .split(",")
                .filter(|x| x.contains("."))
                .map(|x| x.to_owned())
                .collect();
            if !ss.is_empty() {
                return ss;
            }
        }
        return RENDEZVOUS_SERVERS.iter().map(|x| x.to_string()).collect();
    }

    pub fn reset_online() {
        *ONLINE.lock().unwrap() = Default::default();
    }

    pub fn update_latency(host: &str, latency: i64) {
        ONLINE.lock().unwrap().insert(host.to_owned(), latency);
        let mut host = "".to_owned();
        let mut delay = i64::MAX;
        for (tmp_host, tmp_delay) in ONLINE.lock().unwrap().iter() {
            if tmp_delay > &0 && tmp_delay < &delay {
                delay = tmp_delay.clone();
                host = tmp_host.to_string();
            }
        }
        if !host.is_empty() {
            let mut config = CONFIG2.write().unwrap();
            if host != config.rendezvous_server {
                log::debug!("Update rendezvous_server in config to {}", host);
                log::debug!("{:?}", *ONLINE.lock().unwrap());
                config.rendezvous_server = host;
                config.store();
            }
        }
    }

    pub fn set_id(id: &str) {
        let mut config = CONFIG.write().unwrap();
        if id == config.id {
            return;
        }
        config.id = id.into();
        config.store();
    }

    pub fn set_nat_type(nat_type: i32) {
        let mut config = CONFIG2.write().unwrap();
        if nat_type == config.nat_type {
            return;
        }
        config.nat_type = nat_type;
        config.store();
    }

    pub fn get_nat_type() -> i32 {
        CONFIG2.read().unwrap().nat_type
    }

    pub fn set_serial(serial: i32) {
        let mut config = CONFIG2.write().unwrap();
        if serial == config.serial {
            return;
        }
        config.serial = serial;
        config.store();
    }

    pub fn get_serial() -> i32 {
        std::cmp::max(CONFIG2.read().unwrap().serial, SERIAL)
    }

    fn get_auto_id() -> Option<String> {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        return None;
        let mut id = 0u32;
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        if let Ok(Some(ma)) = mac_address::get_mac_address() {
            for x in &ma.bytes()[2..] {
                id = (id << 8) | (*x as u32);
            }
            id = id & 0x1FFFFFFF;
            Some(id.to_string())
        } else {
            None
        }
    }

    pub fn get_auto_password() -> String {
        let mut rng = rand::thread_rng();
        (0..6)
            .map(|_| CHARS[rng.gen::<usize>() % CHARS.len()])
            .collect()
    }

    pub fn get_key_confirmed() -> bool {
        CONFIG.read().unwrap().key_confirmed
    }

    pub fn set_key_confirmed(v: bool) {
        let mut config = CONFIG.write().unwrap();
        if config.key_confirmed == v {
            return;
        }
        config.key_confirmed = v;
        if !v {
            config.keys_confirmed = Default::default();
        }
        config.store();
    }

    pub fn get_host_key_confirmed(host: &str) -> bool {
        if let Some(true) = CONFIG.read().unwrap().keys_confirmed.get(host) {
            true
        } else {
            false
        }
    }

    pub fn set_host_key_confirmed(host: &str, v: bool) {
        if Self::get_host_key_confirmed(host) == v {
            return;
        }
        let mut config = CONFIG.write().unwrap();
        config.keys_confirmed.insert(host.to_owned(), v);
        config.store();
    }

    pub fn set_key_pair(pair: (Vec<u8>, Vec<u8>)) {
        let mut config = CONFIG.write().unwrap();
        if config.key_pair == pair {
            return;
        }
        config.key_pair = pair;
        config.store();
    }

    /// Ecletica: existe par de chaves guardado? Le sem gerar um novo,
    /// (get_key_pair() gera e grava quando o par esta vazio).
    pub fn has_key_pair() -> bool {
        !CONFIG.read().unwrap().key_pair.0.is_empty()
    }

    pub fn get_key_pair() -> (Vec<u8>, Vec<u8>) {
        // lock here to make sure no gen_keypair more than once
        let mut config = CONFIG.write().unwrap();
        if config.key_pair.0.is_empty() {
            let (pk, sk) = sign::gen_keypair();
            config.key_pair = (sk.0.to_vec(), pk.0.into());
            config.store();
        }
        config.key_pair.clone()
    }

    pub fn get_id() -> String {
        let mut id = CONFIG.read().unwrap().id.clone();
        if id.is_empty() {
            if let Some(tmp) = Config::get_auto_id() {
                id = tmp;
                Config::set_id(&id);
            }
        }
        id
    }

    pub fn get_options() -> HashMap<String, String> {
        CONFIG2.read().unwrap().options.clone()
    }

    pub fn set_options(v: HashMap<String, String>) {
        let mut config = CONFIG2.write().unwrap();
        if config.options == v {
            return;
        }
        config.options = v;
        config.store();
    }

    pub fn get_option(k: &str) -> String {
        if let Some(v) = CONFIG2.read().unwrap().options.get(k) {
            v.clone()
        } else {
            "".to_owned()
        }
    }

    pub fn set_option(k: String, v: String) {
        let mut config = CONFIG2.write().unwrap();
        if k == "custom-rendezvous-server" {
            config.rendezvous_server = "".to_owned();
        }
        let v2 = if v.is_empty() { None } else { Some(&v) };
        if v2 != config.options.get(&k) {
            if v2.is_none() {
                config.options.remove(&k);
            } else {
                config.options.insert(k, v);
            }
            config.store();
        }
    }

    pub fn update_id() {
        // to-do: how about if one ip register a lot of ids?
        let id = Self::get_id();
        let mut rng = rand::thread_rng();
        let new_id = rng.gen_range(1_000_000_000..2_000_000_000).to_string();
        Config::set_id(&new_id);
        log::info!("id updated from {} to {}", id, new_id);
    }

    pub fn set_password(password: &str) {
        let mut config = CONFIG.write().unwrap();
        if password == config.password {
            return;
        }
        config.password = password.into();
        config.store();
    }

    pub fn get_password() -> String {
        let mut password = CONFIG.read().unwrap().password.clone();
        if password.is_empty() {
            password = Config::get_auto_password();
            Config::set_password(&password);
        }
        password
    }

    pub fn set_salt(salt: &str) {
        let mut config = CONFIG.write().unwrap();
        if salt == config.salt {
            return;
        }
        config.salt = salt.into();
        config.store();
    }

    pub fn get_salt() -> String {
        let mut salt = CONFIG.read().unwrap().salt.clone();
        if salt.is_empty() {
            salt = Config::get_auto_password();
            Config::set_salt(&salt);
        }
        salt
    }

    pub fn get_size() -> Size {
        CONFIG2.read().unwrap().size
    }

    pub fn set_size(x: i32, y: i32, w: i32, h: i32) {
        let mut config = CONFIG2.write().unwrap();
        let size = (x, y, w, h);
        if size == config.size || size.2 < 300 || size.3 < 300 {
            return;
        }
        config.size = size;
        config.store();
    }

    pub fn set_remote_id(remote_id: &str) {
        let mut config = CONFIG2.write().unwrap();
        if remote_id == config.remote_id {
            return;
        }
        config.remote_id = remote_id.into();
        config.store();
    }

    pub fn get_remote_id() -> String {
        CONFIG2.read().unwrap().remote_id.clone()
    }

    pub fn set_socks(socks: Option<Socks5Server>) {
        let mut config = CONFIG2.write().unwrap();
        if config.socks == socks {
            return;
        }
        config.socks = socks;
        config.store();
    }

    pub fn get_socks() -> Option<Socks5Server> {
        CONFIG2.read().unwrap().socks.clone()
    }

    pub fn get_network_type() -> NetworkType {
        match &CONFIG2.read().unwrap().socks {
            None => NetworkType::Direct,
            Some(_) => NetworkType::ProxySocks,
        }
    }
}

const PEERS: &str = "peers";

impl PeerConfig {
    pub fn load(id: &str) -> PeerConfig {
        let _ = CONFIG.read().unwrap(); // for lock
        match confy::load_path(&Self::path(id)) {
            Ok(config) => config,
            Err(err) => {
                log::error!("Failed to load config: {}", err);
                Default::default()
            }
        }
    }

    pub fn store(&self, id: &str) {
        let _ = CONFIG.read().unwrap(); // for lock
        if let Err(err) = confy::store_path(Self::path(id), self) {
            log::error!("Failed to store config: {}", err);
        }
    }

    pub fn remove(id: &str) {
        fs::remove_file(&Self::path(id)).ok();
    }

    fn path(id: &str) -> PathBuf {
        let path: PathBuf = [PEERS, id].iter().collect();
        Config::path(path).with_extension("toml")
    }

    pub fn peers() -> Vec<(String, SystemTime, PeerConfig)> {
        if let Ok(peers) = Config::path(PEERS).read_dir() {
            if let Ok(peers) = peers
                .map(|res| res.map(|e| e.path()))
                .collect::<Result<Vec<_>, _>>()
            {
                let mut peers: Vec<_> = peers
                    .iter()
                    .filter(|p| {
                        p.is_file()
                            && p.extension().map(|p| p.to_str().unwrap_or("")) == Some("toml")
                    })
                    .map(|p| {
                        let t = fs::metadata(p)
                            .map(|m| m.modified().unwrap_or(SystemTime::UNIX_EPOCH))
                            .unwrap_or(SystemTime::UNIX_EPOCH);
                        let id = p
                            .file_stem()
                            .map(|p| p.to_str().unwrap_or(""))
                            .unwrap_or("")
                            .to_owned();
                        let c = PeerConfig::load(&id);
                        if c.info.platform.is_empty() {
                            fs::remove_file(&p).ok();
                        }
                        (id, t, c)
                    })
                    .filter(|p| !p.2.info.platform.is_empty())
                    .collect();
                peers.sort_unstable_by(|a, b| b.1.cmp(&a.1));
                return peers;
            }
        }
        Default::default()
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Fav {
    #[serde(default)]
    pub peers: Vec<String>,
}

impl Fav {
    pub fn load() -> Fav {
        let _ = CONFIG.read().unwrap(); // for lock
        match confy::load_path(&Config::file_("_fav")) {
            Ok(fav) => fav,
            Err(err) => {
                log::error!("Failed to load fav: {}", err);
                Default::default()
            }
        }
    }

    pub fn store(peers: Vec<String>) {
        let f = Fav { peers };
        if let Err(err) = confy::store_path(Config::file_("_fav"), f) {
            log::error!("Failed to store fav: {}", err);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_serialize() {
        let cfg: Config = Default::default();
        let res = toml::to_string_pretty(&cfg);
        assert!(res.is_ok());
        let cfg: PeerConfig = Default::default();
        let res = toml::to_string_pretty(&cfg);
        assert!(res.is_ok());
    }
}
