// Do not show a console window on Windows
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::sync::Mutex;
use tauri::{
    api::{file, notification::Notification},
    Manager, State,
};
extern crate chrono;
extern crate timer;

#[derive(Serialize, Deserialize, Clone)]
struct Credentials {
    username: String,
    password: String,
}

#[derive(Clone)]
struct ConnectState {
    login: bool,
    logout: bool,
    login_endpoint: String,
    credentials: Credentials,
}

fn save_creds(creds: Credentials, save_file: &std::path::Path) {
    let mut file = std::fs::File::create(&save_file).unwrap();
    write!(&mut file, "{}", serde_json::to_string(&creds).unwrap()).unwrap();
}

fn load_creds(save_file: &std::path::Path) -> Result<Credentials, String> {
    let creds_string = file::read_string(save_file);
    if creds_string.is_ok() {
        let creds: Credentials = serde_json::from_str(&creds_string.unwrap()).unwrap();
        return Ok(creds);
    } else {
        return Err("Credentials not saved".to_string());
    }
}

fn logout_campnet(
    client: reqwest::blocking::Client,
    creds: Credentials,
    login_endpoint: String,
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let body: String = format!(
        "mode=193&username={}&a={}&producttype=1",
        creds.username,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    return client
        .post(login_endpoint + "/logout.xml")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Content-Length", body.chars().count())
        .body(body)
        .send();
}

fn login_campnet(
    client: reqwest::blocking::Client,
    creds: Credentials,
    login_endpoint: String,
) -> Result<reqwest::blocking::Response, reqwest::Error> {
    let body: String = format!(
        "mode=191&username={}&password={}&a={}&producttype=1",
        creds.username,
        creds.password,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    return client
        .post(login_endpoint.to_owned() + "/login.xml")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Content-Length", body.chars().count())
        .body(body)
        .send();
}

fn connect_campnet(app: tauri::AppHandle, app_state: State<Mutex<ConnectState>>) {
    let tray_handle = app.tray_handle();
    let resources_resolver = app.path_resolver();
    let active_icon_path = resources_resolver
        .resolve_resource("resources/icons/active.png")
        .unwrap();
    let passive_icon_path = resources_resolver
        .resolve_resource("resources/icons/passive.png")
        .unwrap();
    let credentials = app_state.lock().unwrap().credentials.to_owned();
    let client = reqwest::blocking::Client::new();
    if app_state.lock().unwrap().login {
        let campnet_status = client
            .head(app_state.lock().unwrap().login_endpoint.to_owned())
            .send();
        if campnet_status.is_ok() {
            let login_status = client.head("https://www.google.com").send();
            if login_status.is_err() {
                let res = login_campnet(
                    client,
                    credentials,
                    app_state.lock().unwrap().login_endpoint.to_string(),
                );
                if res.is_ok() {
                    let res_body: String = res.unwrap().text().unwrap();
                    if res_body.contains("LIVE") {
                        Notification::new("com.riskycase.autocampnet")
                            .title("Connected to Campnet!")
                            .body("Logged in successfully to BPGC network")
                            .show()
                            .unwrap();
                        tray_handle
                            .set_icon(tauri::Icon::File(active_icon_path))
                            .unwrap();
                    } else if res_body.contains("failed") {
                        Notification::new("com.riskycase.autocampnet")
                            .title("Could not connect to Campnet!")
                            .body("Incorrect credentials were provided")
                            .show()
                            .unwrap();
                        app_state.lock().unwrap().login = false;
                        tray_handle
                            .set_icon(tauri::Icon::File(passive_icon_path))
                            .unwrap();
                    } else if res_body.contains("exceeded") {
                        Notification::new("com.riskycase.autocampnet")
                            .title("Could not connect to Campnet!")
                            .body("Daily data limit exceeded on credentials")
                            .show()
                            .unwrap();
                        app_state.lock().unwrap().login = false;
                        tray_handle
                            .set_icon(tauri::Icon::File(passive_icon_path))
                            .unwrap();
                    } else {
                        Notification::new("com.riskycase.autocampnet")
                            .title("Could not to Campnet!")
                            .body("There was an issue with the login attempt")
                            .show()
                            .unwrap();
                        app_state.lock().unwrap().login = false;
                        tray_handle
                            .set_icon(tauri::Icon::File(passive_icon_path))
                            .unwrap();
                    }
                }
            } else {
                tray_handle
                    .set_icon(tauri::Icon::File(active_icon_path))
                    .unwrap();
            }
        }
    } else if app_state.lock().unwrap().logout {
        let res = logout_campnet(
            client,
            credentials,
            app_state.lock().unwrap().login_endpoint.to_string(),
        );
        if res.is_ok() {
            let res_body: String = res.unwrap().text().unwrap();
            if res_body.contains("LOGIN") {
                Notification::new("com.riskycase.autocampnet")
                    .title("Logged out of Campnet")
                    .show()
                    .unwrap();
            }
        }
        app_state.lock().unwrap().logout = false;
    }

    let callback_timer = timer::Timer::new();
    let _callback_gaurd =
        callback_timer.schedule_with_delay(chrono::Duration::milliseconds(2500), move || {
            connect_campnet(app.app_handle(), app.state::<Mutex<ConnectState>>());
        });
    std::thread::sleep(std::time::Duration::from_millis(3000));
}

fn main() {
    let tray_menu = tauri::SystemTrayMenu::new()
        .add_item(tauri::CustomMenuItem::new("show", "Show window"))
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(tauri::CustomMenuItem::new("reconnect", "Force reconnect"))
        .add_item(tauri::CustomMenuItem::new("logout", "Logout"))
        .add_item(tauri::CustomMenuItem::new("delete", "Delete credentials"))
        .add_native_item(tauri::SystemTrayMenuItem::Separator)
        .add_item(tauri::CustomMenuItem::new("quit", "Quit"));
    let system_tray = tauri::SystemTray::new().with_menu(tray_menu);
    tauri::Builder::default()
        .setup(|app: &mut tauri::App| {
            app.manage(Mutex::new(ConnectState {
                login: false,
                logout: false,
                login_endpoint: String::new(),
                credentials: Credentials {
                    username: "".to_string(),
                    password: "".to_string(),
                },
            }));
            let file_path = app
                .path_resolver()
                .app_config_dir()
                .unwrap()
                .join("credentials.json");
            let creds = load_creds(&file_path);
            let app_state: State<Mutex<ConnectState>> = app.state::<Mutex<ConnectState>>();
            if creds.is_ok() {
                app_state.lock().unwrap().login_endpoint =
                    String::from("https://campnet.bits-goa.ac.in:8090");
                app_state.lock().unwrap().login = true;
                app_state.lock().unwrap().credentials = creds.unwrap();
                connect_campnet(app.app_handle(), app_state.to_owned());
            } else {
                app.get_window("main").unwrap().show().unwrap();
            }
            let app_handle_save = app.app_handle();
            app.listen_global("save", move |event: tauri::Event| {
                let creds: Credentials = serde_json::from_str(event.payload().unwrap()).unwrap();
                let app_state = app_handle_save.state::<Mutex<ConnectState>>();
                app_state.lock().unwrap().credentials = creds.clone();
                save_creds(creds, &file_path);
                app_state.lock().unwrap().login = true;
                Notification::new("com.riskycase.autocampnet")
                    .title("Credentials saved to disk")
                    .body("App will try to login to campnet whenever available")
                    .show()
                    .unwrap();
            });
            let app_handle_minimise = app.app_handle();
            app.listen_global("minimise", move |_event: tauri::Event| {
                app_handle_minimise
                    .get_window("main")
                    .unwrap()
                    .hide()
                    .unwrap();
            });
            std::fs::create_dir_all(app.path_resolver().app_config_dir().unwrap()).unwrap();
            connect_campnet(app.app_handle(), app_state);
            Ok(())
        })
        .system_tray(system_tray)
        .on_system_tray_event(|app: &tauri::AppHandle, event| match event {
            tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "show" => {
                    let app_state = app.state::<Mutex<ConnectState>>();
                    let window: tauri::Window = app.get_window("main").unwrap();
                    window
                        .emit("credentials", app_state.lock().unwrap().credentials.clone())
                        .unwrap();
                    window.show().unwrap();
                    window.unminimize().unwrap();
                    window.set_focus().unwrap();
                }
                "logout" => {
                    let app_state = app.state::<Mutex<ConnectState>>();
                    app_state.lock().unwrap().logout = true;
                    app_state.lock().unwrap().login = false;
                    app.tray_handle()
                        .set_icon(tauri::Icon::File(
                            app.path_resolver()
                                .resolve_resource("resources/icons/passive.png")
                                .unwrap(),
                        ))
                        .unwrap();
                }
                "reconnect" => {
                    let app_state = app.state::<Mutex<ConnectState>>();
                    let creds = app_state.lock().unwrap().credentials.to_owned();
                    if (creds.username.len() == 0) | (creds.password.len() == 0) {
                        let window: tauri::Window = app.get_window("main").unwrap();
                        window.show().unwrap();
                    } else {
                        if app_state.lock().unwrap().login {
                            connect_campnet(app.app_handle(), app_state);
                        }
                        app.state::<Mutex<ConnectState>>().lock().unwrap().login = true;
                    }
                }
                "delete" => {
                    let file_path = app
                        .path_resolver()
                        .app_config_dir()
                        .unwrap()
                        .join("credentials.json");
                    std::fs::remove_file(&file_path).unwrap();
                    let app_state = app.state::<Mutex<ConnectState>>();
                    app_state.lock().unwrap().login = false;
                    app_state.lock().unwrap().credentials = Credentials {
                        username: "".to_owned(),
                        password: "".to_owned(),
                    };
                    let window: tauri::Window = app.get_window("main").unwrap();
                    window
                        .emit("credentials", app_state.lock().unwrap().credentials.clone())
                        .unwrap();
                    window.show().unwrap();
                }
                _ => {}
            },
            tauri::SystemTrayEvent::LeftClick {
                tray_id: _,
                position: _,
                size: _,
                ..
            } => {
                let app_state = app.state::<Mutex<ConnectState>>();
                let window: tauri::Window = app.get_window("main").unwrap();
                window
                    .emit("credentials", app_state.lock().unwrap().credentials.clone())
                    .unwrap();
                window.show().unwrap();
                window.unminimize().unwrap();
                window.set_focus().unwrap();
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
