use std::{
    collections::HashMap,
    iter::FromIterator,
    sync::{Arc, Mutex},
};

use sciter::Value;

use hbb_common::{
    allow_err,
    config::{LocalConfig, PeerConfig},
    log,
};

#[cfg(not(feature = "flutter"))]
use crate::ui_session_interface::Session;
use crate::{common::get_app_name, ipc, ui_interface::*};

mod cm;
#[cfg(feature = "inline")]
pub mod inline;
pub mod remote;

#[allow(dead_code)]
type Status = (i32, bool, i64, String);

lazy_static::lazy_static! {
    // stupid workaround for https://sciter.com/forums/topic/crash-on-latest-tis-mac-sdk-sometimes/
    static ref STUPID_VALUES: Mutex<Vec<Arc<Vec<Value>>>> = Default::default();
}

#[cfg(not(feature = "flutter"))]
lazy_static::lazy_static! {
    pub static ref CUR_SESSION: Arc<Mutex<Option<Session<remote::SciterHandler>>>> = Default::default();
}

struct UIHostHandler;

pub fn start(args: &mut [String]) {
    #[cfg(target_os = "macos")]
    crate::platform::delegate::show_dock();
    #[cfg(all(target_os = "linux", feature = "inline"))]
    {
        let app_dir = std::env::var("APPDIR").unwrap_or("".to_string());
        let mut so_path = "/usr/share/rustdesk/libsciter-gtk.so".to_owned();
        for (prefix, dir) in [
            ("", "/usr"),
            ("", "/app"),
            (&app_dir, "/usr"),
            (&app_dir, "/app"),
        ]
        .iter()
        {
            let path = format!("{prefix}{dir}/share/rustdesk/libsciter-gtk.so");
            if std::path::Path::new(&path).exists() {
                so_path = path;
                break;
            }
        }
        sciter::set_library(&so_path).ok();
    }
    #[cfg(windows)]
    // Check if there is a sciter.dll nearby.
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            let sciter_dll_path = parent.join("sciter.dll");
            if sciter_dll_path.exists() {
                // Try to set the sciter dll.
                let p = sciter_dll_path.to_string_lossy().to_string();
                log::debug!("Found dll:{}, \n {:?}", p, sciter::set_library(&p));
            }
        }
    }
    // https://github.com/c-smile/sciter-sdk/blob/master/include/sciter-x-types.h
    // https://github.com/rustdesk/rustdesk/issues/132#issuecomment-886069737
    #[cfg(windows)]
    allow_err!(sciter::set_options(sciter::RuntimeOptions::GfxLayer(
        sciter::GFX_LAYER::WARP
    )));
    use sciter::SCRIPT_RUNTIME_FEATURES::*;
    allow_err!(sciter::set_options(sciter::RuntimeOptions::ScriptFeatures(
        ALLOW_FILE_IO as u8 | ALLOW_SOCKET_IO as u8 | ALLOW_EVAL as u8 | ALLOW_SYSINFO as u8
    )));
    let mut frame = sciter::WindowBuilder::main_window().create();
    #[cfg(windows)]
    allow_err!(sciter::set_options(sciter::RuntimeOptions::UxTheming(true)));
    frame.set_title(&crate::get_app_name());
    #[cfg(target_os = "macos")]
    crate::platform::delegate::make_menubar(frame.get_host(), args.is_empty());
    #[cfg(windows)]
    crate::platform::try_set_window_foreground(frame.get_hwnd() as _);
    let page;
    if args.len() > 1 && args[0] == "--play" {
        args[0] = "--connect".to_owned();
        let path: std::path::PathBuf = (&args[1]).into();
        let id = path
            .file_stem()
            .map(|p| p.to_str().unwrap_or(""))
            .unwrap_or("")
            .to_owned();
        args[1] = id;
    }
    if args.is_empty() {
        std::thread::spawn(move || check_zombie());
        crate::common::check_software_update();
        frame.event_handler(UI {});
        frame.sciter_handler(UIHostHandler {});
        page = "index.html";
        // Start pulse audio local server.
        #[cfg(target_os = "linux")]
        std::thread::spawn(crate::ipc::start_pa);
    } else if args[0] == "--install" {
        frame.event_handler(UI {});
        frame.sciter_handler(UIHostHandler {});
        page = "install.html";
    } else if args[0] == "--cm" {
        frame.register_behavior("connection-manager", move || {
            Box::new(cm::SciterConnectionManager::new())
        });
        page = "cm.html";
        *cm::HIDE_CM.lock().unwrap() = crate::ipc::get_config("hide_cm")
            .ok()
            .flatten()
            .unwrap_or_default()
            == "true";
    } else if (args[0] == "--connect"
        || args[0] == "--file-transfer"
        || args[0] == "--port-forward"
        || args[0] == "--rdp")
        && args.len() > 1
    {
        #[cfg(windows)]
        {
            let hw = frame.get_host().get_hwnd();
            crate::platform::windows::enable_lowlevel_keyboard(hw as _);
        }
        let mut iter = args.iter();
        let Some(cmd) = iter.next() else {
            log::error!("Failed to get cmd arg");
            return;
        };
        let cmd = cmd.to_owned();
        let Some(id) = iter.next() else {
            log::error!("Failed to get id arg");
            return;
        };
        let id = id.to_owned();
        let pass = iter.next().unwrap_or(&"".to_owned()).clone();
        let args: Vec<String> = iter.map(|x| x.clone()).collect();
        frame.set_title(&id);
        frame.register_behavior("native-remote", move || {
            let handler =
                remote::SciterSession::new(cmd.clone(), id.clone(), pass.clone(), args.clone());
            #[cfg(not(feature = "flutter"))]
            {
                *CUR_SESSION.lock().unwrap() = Some(handler.inner());
            }
            Box::new(handler)
        });
        page = "remote.html";
    } else {
        log::error!("Wrong command: {:?}", args);
        return;
    }
    #[cfg(feature = "inline")]
    {
        let html = if page == "index.html" {
            inline::get_index()
        } else if page == "cm.html" {
            inline::get_cm()
        } else if page == "install.html" {
            inline::get_install()
        } else {
            inline::get_remote()
        };
        frame.load_html(html.as_bytes(), Some(page));
    }
    #[cfg(not(feature = "inline"))]
    frame.load_file(&format!(
        "file://{}/src/ui/{}",
        std::env::current_dir()
            .map(|c| c.display().to_string())
            .unwrap_or("".to_owned()),
        page
    ));
    let hide_cm = *cm::HIDE_CM.lock().unwrap();
    if !args.is_empty() && args[0] == "--cm" && hide_cm {
        // run_app calls expand(show) + run_loop, we use collapse(hide) + run_loop instead to create a hidden window
        frame.collapse(true);
        frame.run_loop();
        return;
    }
    frame.run_app();
}

struct UI {}

impl UI {
    fn recent_sessions_updated(&self) -> bool {
        recent_sessions_updated()
    }

    fn get_id(&self) -> String {
        ipc::get_id()
    }

    fn temporary_password(&mut self) -> String {
        temporary_password()
    }

    fn update_temporary_password(&self) {
        update_temporary_password()
    }

    fn set_permanent_password(&self, password: String) {
        let _ = set_permanent_password_with_result(password);
    }

    fn is_local_permanent_password_set(&self) -> bool {
        is_local_permanent_password_set()
    }

    fn is_permanent_password_set(&self) -> bool {
        is_permanent_password_set()
    }

    fn get_remote_id(&mut self) -> String {
        LocalConfig::get_remote_id()
    }

    fn set_remote_id(&mut self, id: String) {
        LocalConfig::set_remote_id(&id);
    }

    fn goto_install(&mut self) {
        goto_install();
    }

    fn install_me(&mut self, _options: String, _path: String) {
        install_me(_options, _path, false, false);
    }

    fn update_me(&self, _path: String) {
        update_me(_path);
    }

    fn run_without_install(&self) {
        run_without_install();
    }

    fn show_run_without_install(&self) -> bool {
        show_run_without_install()
    }

    fn get_license(&self) -> String {
        get_license()
    }

    fn get_option(&self, key: String) -> String {
        get_option(key)
    }

    fn get_local_option(&self, key: String) -> String {
        get_local_option(key)
    }

    fn set_local_option(&self, key: String, value: String) {
        set_local_option(key, value);
    }

    fn peer_has_password(&self, id: String) -> bool {
        peer_has_password(id)
    }

    fn forget_password(&self, id: String) {
        forget_password(id)
    }

    fn get_peer_option(&self, id: String, name: String) -> String {
        get_peer_option(id, name)
    }

    fn set_peer_option(&self, id: String, name: String, value: String) {
        set_peer_option(id, name, value)
    }

    fn using_public_server(&self) -> bool {
        crate::using_public_server()
    }

    fn is_incoming_only(&self) -> bool {
        hbb_common::config::is_incoming_only()
    }

    pub fn is_outgoing_only(&self) -> bool {
        hbb_common::config::is_outgoing_only()
    }

    pub fn is_custom_client(&self) -> bool {
        crate::common::is_custom_client()
    }

    pub fn is_disable_settings(&self) -> bool {
        hbb_common::config::is_disable_settings()
    }

    pub fn is_disable_account(&self) -> bool {
        hbb_common::config::is_disable_account()
    }

    pub fn is_disable_installation(&self) -> bool {
        hbb_common::config::is_disable_installation()
    }

    pub fn is_disable_ab(&self) -> bool {
        hbb_common::config::is_disable_ab()
    }

    fn get_options(&self) -> Value {
        let hashmap: HashMap<String, String> =
            serde_json::from_str(&get_options()).unwrap_or_default();
        let mut m = Value::map();
        for (k, v) in hashmap {
            m.set_item(k, v);
        }
        m
    }

    fn test_if_valid_server(&self, host: String, test_with_proxy: bool) -> String {
        test_if_valid_server(host, test_with_proxy)
    }

    fn get_sound_inputs(&self) -> Value {
        Value::from_iter(get_sound_inputs())
    }

    fn set_options(&self, v: Value) {
        let mut m = HashMap::new();
        for (k, v) in v.items() {
            if let Some(k) = k.as_string() {
                if let Some(v) = v.as_string() {
                    if !v.is_empty() {
                        m.insert(k, v);
                    }
                }
            }
        }
        set_options(m);
    }

    fn set_option(&self, key: String, value: String) {
        set_option(key, value);
    }

    fn install_path(&mut self) -> String {
        install_path()
    }

    fn install_options(&self) -> String {
        install_options()
    }

    fn get_socks(&self) -> Value {
        Value::from_iter(get_socks())
    }

    fn set_socks(&self, proxy: String, username: String, password: String) {
        set_socks(proxy, username, password)
    }

    fn is_installed(&self) -> bool {
        is_installed()
    }

    fn get_supported_privacy_mode_impls(&self) -> String {
        serde_json::to_string(&crate::privacy_mode::get_supported_privacy_mode_impl())
            .unwrap_or_default()
    }

    fn is_root(&self) -> bool {
        is_root()
    }

    fn is_release(&self) -> bool {
        #[cfg(not(debug_assertions))]
        return true;
        #[cfg(debug_assertions)]
        return false;
    }

    fn is_share_rdp(&self) -> bool {
        is_share_rdp()
    }

    fn set_share_rdp(&self, _enable: bool) {
        set_share_rdp(_enable);
    }

    fn is_installed_lower_version(&self) -> bool {
        is_installed_lower_version()
    }

    fn closing(&mut self, x: i32, y: i32, w: i32, h: i32) {
        crate::server::input_service::fix_key_down_timeout_at_exit();
        LocalConfig::set_size(x, y, w, h);
    }

    fn get_size(&mut self) -> Value {
        let s = LocalConfig::get_size();
        let mut v = Vec::new();
        v.push(s.0);
        v.push(s.1);
        v.push(s.2);
        v.push(s.3);
        Value::from_iter(v)
    }

    fn get_mouse_time(&self) -> f64 {
        get_mouse_time()
    }

    fn check_mouse_time(&self) {
        check_mouse_time()
    }

    fn get_connect_status(&mut self) -> Value {
        let mut v = Value::array(0);
        let x = get_connect_status();
        v.push(x.status_num);
        v.push(x.key_confirmed);
        v.push(x.id);
        v
    }

    #[inline]
    fn get_peer_value(id: String, p: PeerConfig) -> Value {
        let values = vec![
            id,
            p.info.username.clone(),
            p.info.hostname.clone(),
            p.info.platform.clone(),
            p.options.get("alias").unwrap_or(&"".to_owned()).to_owned(),
        ];
        Value::from_iter(values)
    }

    fn get_peer(&self, id: String) -> Value {
        let c = get_peer(id.clone());
        Self::get_peer_value(id, c)
    }

    fn get_fav(&self) -> Value {
        Value::from_iter(get_fav())
    }

    fn store_fav(&self, fav: Value) {
        let mut tmp = vec![];
        fav.values().for_each(|v| {
            if let Some(v) = v.as_string() {
                if !v.is_empty() {
                    tmp.push(v);
                }
            }
        });
        store_fav(tmp);
    }

    fn get_recent_sessions(&mut self) -> Value {
        // to-do: limit number of recent sessions, and remove old peer file
        let peers: Vec<Value> = PeerConfig::peers(None)
            .drain(..)
            .map(|p| Self::get_peer_value(p.0, p.2))
            .collect();
        Value::from_iter(peers)
    }

    fn get_icon(&mut self) -> String {
        get_icon()
    }

    fn remove_peer(&mut self, id: String) {
        PeerConfig::remove(&id);
    }

    fn remove_discovered(&mut self, id: String) {
        remove_discovered(id);
    }

    fn send_wol(&mut self, id: String) {
        crate::lan::send_wol(id)
    }

    fn new_remote(&mut self, id: String, remote_type: String, force_relay: bool) {
        new_remote(id, remote_type, force_relay)
    }

    fn is_process_trusted(&mut self, _prompt: bool) -> bool {
        is_process_trusted(_prompt)
    }

    fn is_can_screen_recording(&mut self, _prompt: bool) -> bool {
        is_can_screen_recording(_prompt)
    }

    fn is_installed_daemon(&mut self, _prompt: bool) -> bool {
        is_installed_daemon(_prompt)
    }

    fn get_error(&mut self) -> String {
        get_error()
    }

    fn is_login_wayland(&mut self) -> bool {
        is_login_wayland()
    }

    fn current_is_wayland(&mut self) -> bool {
        current_is_wayland()
    }

    fn get_software_update_url(&self) -> String {
        crate::SOFTWARE_UPDATE_URL.lock().unwrap().clone()
    }

    fn get_new_version(&self) -> String {
        get_new_version()
    }

    fn get_version(&self) -> String {
        get_version()
    }

    fn get_fingerprint(&self) -> String {
        get_fingerprint()
    }

    fn get_app_name(&self) -> String {
        get_app_name()
    }

    fn get_software_ext(&self) -> String {
        #[cfg(windows)]
        let p = "exe";
        #[cfg(target_os = "macos")]
        let p = "dmg";
        #[cfg(target_os = "linux")]
        let p = "deb";
        p.to_owned()
    }

    fn get_software_store_path(&self) -> String {
        let mut p = std::env::temp_dir();
        let name = crate::SOFTWARE_UPDATE_URL
            .lock()
            .unwrap()
            .split("/")
            .last()
            .map(|x| x.to_owned())
            .unwrap_or(crate::get_app_name());
        p.push(name);
        format!("{}.{}", p.to_string_lossy(), self.get_software_ext())
    }

    fn create_shortcut(&self, _id: String) {
        #[cfg(windows)]
        create_shortcut(_id)
    }

    fn discover(&self) {
        std::thread::spawn(move || {
            allow_err!(crate::lan::discover());
        });
    }

    fn get_lan_peers(&self) -> String {
        // let peers = get_lan_peers()
        //     .into_iter()
        //     .map(|mut peer| {
        //         (
        //             peer.remove("id").unwrap_or_default(),
        //             peer.remove("username").unwrap_or_default(),
        //             peer.remove("hostname").unwrap_or_default(),
        //             peer.remove("platform").unwrap_or_default(),
        //         )
        //     })
        //     .collect::<Vec<(String, String, String, String)>>();
        serde_json::to_string(&get_lan_peers()).unwrap_or_default()
    }

    fn get_uuid(&self) -> String {
        get_uuid()
    }

    fn open_url(&self, url: String) {
        #[cfg(windows)]
        let p = "explorer";
        #[cfg(target_os = "macos")]
        let p = "open";
        #[cfg(target_os = "linux")]
        let p = if std::path::Path::new("/usr/bin/firefox").exists() {
            "firefox"
        } else {
            "xdg-open"
        };
        allow_err!(std::process::Command::new(p).arg(url).spawn());
    }

    fn change_id(&self, id: String) {
        reset_async_job_status();
        let old_id = self.get_id();
        change_id_shared(id, old_id);
    }

    fn http_request(&self, url: String, method: String, body: Option<String>, header: String) {
        http_request(url, method, body, header)
    }

    fn post_request(&self, url: String, body: String, header: String) {
        post_request(url, body, header)
    }

    fn is_ok_change_id(&self) -> bool {
        hbb_common::machine_uid::get().is_ok()
    }

    fn get_async_job_status(&self) -> String {
        get_async_job_status()
    }

    fn get_http_status(&self, url: String) -> Option<String> {
        get_async_http_status(url)
    }

    fn t(&self, name: String) -> String {
        crate::client::translate(name)
    }

    fn is_xfce(&self) -> bool {
        crate::platform::is_xfce()
    }

    fn get_api_server(&self) -> String {
        get_api_server()
    }

    fn has_hwcodec(&self) -> bool {
        has_hwcodec()
    }

    fn has_vram(&self) -> bool {
        has_vram()
    }

    fn get_langs(&self) -> String {
        get_langs()
    }

    fn video_save_directory(&self, root: bool) -> String {
        video_save_directory(root)
    }

    fn handle_relay_id(&self, id: String) -> String {
        handle_relay_id(&id).to_owned()
    }

    fn get_login_device_info(&self) -> String {
        get_login_device_info_json()
    }

    fn support_remove_wallpaper(&self) -> bool {
        support_remove_wallpaper()
    }

    fn has_valid_2fa(&self) -> bool {
        has_valid_2fa()
    }

    fn generate2fa(&self) -> String {
        generate2fa()
    }

    pub fn verify2fa(&self, code: String) -> bool {
        verify2fa(code)
    }

    fn verify_login(&self, raw: String, id: String) -> bool {
        crate::verify_login(&raw, &id)
    }

    fn generate_2fa_img_src(&self, data: String) -> String {
        let v = qrcode_generator::to_png_to_vec(data, qrcode_generator::QrCodeEcc::Low, 128)
            .unwrap_or_default();
        let s = hbb_common::sodiumoxide::base64::encode(
            v,
            hbb_common::sodiumoxide::base64::Variant::Original,
        );
        format!("data:image/png;base64,{s}")
    }

    pub fn check_hwcodec(&self) {
        check_hwcodec()
    }

    fn is_option_fixed(&self, key: String) -> bool {
        crate::ui_interface::is_option_fixed(&key)
    }

    fn get_builtin_option(&self, key: String) -> String {
        crate::ui_interface::get_builtin_option(&key)
    }

    fn is_remote_modify_enabled_by_control_permissions(&self) -> String {
        match crate::ui_interface::is_remote_modify_enabled_by_control_permissions() {
            Some(true) => "true",
            Some(false) => "false",
            None => "",
        }
        .to_string()
    }
}

impl sciter::EventHandler for UI {
    sciter::dispatch_script_call! {
        fn t(String);
        fn get_api_server();
        fn is_xfce();
        fn using_public_server();
        fn is_custom_client();
        fn is_outgoing_only();
        fn is_incoming_only();
        fn is_disable_settings();
        fn is_disable_account();
        fn is_disable_installation();
        fn is_disable_ab();
        fn get_id();
        fn temporary_password();
        fn update_temporary_password();
        fn set_permanent_password(String);
        fn is_local_permanent_password_set();
        fn is_permanent_password_set();
        fn get_remote_id();
        fn set_remote_id(String);
        fn closing(i32, i32, i32, i32);
        fn get_size();
        fn new_remote(String, String, bool);
        fn send_wol(String);
        fn remove_peer(String);
        fn remove_discovered(String);
        fn get_connect_status();
        fn get_mouse_time();
        fn check_mouse_time();
        fn get_recent_sessions();
        fn get_peer(String);
        fn get_fav();
        fn store_fav(Value);
        fn recent_sessions_updated();
        fn get_icon();
        fn install_me(String, String);
        fn is_installed();
        fn get_supported_privacy_mode_impls();
        fn is_root();
        fn is_release();
        fn set_socks(String, String, String);
        fn get_socks();
        fn is_share_rdp();
        fn set_share_rdp(bool);
        fn is_installed_lower_version();
        fn install_path();
        fn install_options();
        fn goto_install();
        fn is_process_trusted(bool);
        fn is_can_screen_recording(bool);
        fn is_installed_daemon(bool);
        fn get_error();
        fn is_login_wayland();
        fn current_is_wayland();
        fn get_options();
        fn get_option(String);
        fn get_local_option(String);
        fn set_local_option(String, String);
        fn get_peer_option(String, String);
        fn peer_has_password(String);
        fn forget_password(String);
        fn set_peer_option(String, String, String);
        fn get_license();
        fn test_if_valid_server(String, bool);
        fn get_sound_inputs();
        fn set_options(Value);
        fn set_option(String, String);
        fn get_software_update_url();
        fn get_new_version();
        fn get_version();
        fn get_fingerprint();
        fn update_me(String);
        fn show_run_without_install();
        fn run_without_install();
        fn get_app_name();
        fn get_software_store_path();
        fn get_software_ext();
        fn open_url(String);
        fn change_id(String);
        fn get_async_job_status();
        fn post_request(String, String, String);
        fn is_ok_change_id();
        fn create_shortcut(String);
        fn discover();
        fn get_lan_peers();
        fn get_uuid();
        fn has_hwcodec();
        fn has_vram();
        fn get_langs();
        fn video_save_directory(bool);
        fn handle_relay_id(String);
        fn get_login_device_info();
        fn support_remove_wallpaper();
        fn has_valid_2fa();
        fn generate2fa();
        fn generate_2fa_img_src(String);
        fn verify2fa(String);
        fn check_hwcodec();
        fn verify_login(String, String);
        fn is_option_fixed(String);
        fn get_builtin_option(String);
        fn is_remote_modify_enabled_by_control_permissions();
    }
}

impl sciter::host::HostHandler for UIHostHandler {
    fn on_graphics_critical_failure(&mut self) {
        log::error!("Critical rendering error: e.g. DirectX gfx driver error. Most probably bad gfx drivers.");
    }
}

#[cfg(not(target_os = "linux"))]
fn get_sound_inputs() -> Vec<String> {
    let mut out = Vec::new();
    use cpal::traits::{DeviceTrait, HostTrait};
    let host = cpal::default_host();
    if let Ok(devices) = host.devices() {
        for device in devices {
            if device.default_input_config().is_err() {
                continue;
            }
            if let Ok(name) = device.name() {
                out.push(name);
            }
        }
    }
    out
}

#[cfg(target_os = "linux")]
fn get_sound_inputs() -> Vec<String> {
    crate::platform::linux::get_pa_sources()
        .drain(..)
        .map(|x| x.1)
        .collect()
}

// sacrifice some memory
pub fn value_crash_workaround(values: &[Value]) -> Arc<Vec<Value>> {
    let persist = Arc::new(values.to_vec());
    STUPID_VALUES.lock().unwrap().push(persist.clone());
    persist
}

pub fn get_icon() -> String {
    // 128x128
    #[cfg(target_os = "macos")]
    // 128x128 on 160x160 canvas, then shrink to 128, mac looks better with padding
    {
        "data:image/png;base64,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".into()
    }
    #[cfg(not(target_os = "macos"))] // 128x128 no padding
    {
        "data:image/png;base64,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".into()
    }
}
