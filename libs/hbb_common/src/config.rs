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
pub const RENDEZVOUS_TIMEOUT: u64 = 12_000;
pub const CONNECT_TIMEOUT: u64 = 18_000;
pub const COMPRESS_LEVEL: i32 = 3;
const SERIAL: i32 = 1;
// 128x128
#[cfg(target_os = "macos")] // 128x128 on 160x160 canvas, then shrink to 128, mac looks better with padding
pub const ICON: &str = "data:image/png;base64,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";
#[cfg(windows)] // windows, 32x32, bigger very ugly after shrink
pub const ICON: &str = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAIAAAD8GO2jAAACDklEQVR4nO1Uy27TUBA9c23HdkA0bVAlyo4Nj0YgRaJLEOUD+ABAYoHgOyoqfqX8AqjsS5HaUDYlWUVqkZBoIpo0D9mew8JKSR0nMS3scna+M55z5sy9A8www0Uh4wLGGJGx0QlQVZLTaM9VOr1UanWSy6V7Kyv3wzDMTqaqvu9/2tra26sYY1QVgJ1IigM3by2vvVl3c44qszcTRdGVublv1RqGPEgSxIHV1ceXL+UbjYZlWRmrk3Qc5/Dw+9cvlbibdII44Of9er3e6/Ukg0HxQAVQ4t3GRrN5FJuMwfkoxHFyxLSbMCQeoIgEQYDBCLP++7cwxiROZPTTcvyFG08cf5GMJjwUAKAa2+8eVRr1TRFD6mjK2RkIQC7efpG/9kiDE0hSTrI+QztX6Db307SmEAhIxyt4V8tR0BYq0hT9qQ6KWAx+dX5WABDpyUMEAhC2O6/dH6rBFHMGd6d18L7fOhAx8aynY3RKF8RZmSIgbdvJuTkqp/QACAQisZMEup3OJAIxhqoPHq4+ffZcAAKZlgShVM/zqtXa2/U1ESQegT2QLlQtzC+8fPU673tBEGTfcWEYFovF7e3PAI2xoihKITDGRFFUKt29vrR03Dp2XTfrWyRFTLt98nHzA5CUj1OLBkv0Trlc7vd7IiZLAyQAWrazu7NTq+7/3yUxzlJJJJ3vmpI83c8zzPDP8RucxOYZ5KfsDgAAAABJRU5ErkJggg==";
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

// Eclética (custom client): servidor próprio embutido em tempo de compilação.
pub const RENDEZVOUS_SERVERS: &'static [&'static str] = &["app.ecletico.ai"];
// Config de fábrica da Eclética usada quando a opção ainda não foi gravada.
// Mantém o cliente 32 bits configurado sem precisar injetar .toml depois.
pub const ECLETICA_RS_PUB_KEY: &str = "xaLY1y7ytr8vqykrRyukSTeg0zIsqzDscsPAi5tVlYY=";
pub const ECLETICA_API_SERVER: &str = "https://app.ecletico.ai/rustdesk";
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
        let name = format!("{}{}", APP_NAME, suffix);
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
        if let Some(project) = ProjectDirs::from("", org, APP_NAME) {
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
        let v = CONFIG2
            .read()
            .unwrap()
            .options
            .get(k)
            .cloned()
            .unwrap_or_default();
        if !v.is_empty() {
            return v;
        }
        // Config de fábrica da Eclética: aplicada quando a opção ainda não foi
        // gravada, para o cliente já nascer apontando para o nosso servidor
        // (sem injeção de .toml depois da instalação).
        match k {
            "custom-rendezvous-server" | "relay-server" => RENDEZVOUS_SERVERS[0].to_owned(),
            "api-server" => ECLETICA_API_SERVER.to_owned(),
            "key" => ECLETICA_RS_PUB_KEY.to_owned(),
            _ => "".to_owned(),
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
