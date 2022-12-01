// Do not show a console window on Windows
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

static LOGIN_ENDPOINT: &str = "https://campnet.bits-goa.ac.in:8090";

use serde::{Deserialize, Serialize};
use std::io::Write;
use tauri::{
    api::{file, notification::Notification},
    Manager,
};
extern crate chrono;
extern crate timer;

#[derive(Serialize, Deserialize, Clone)]
struct Credentials {
    username: String,
    password: String,
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
    creds: Credentials,
    client: reqwest::blocking::Client,
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
        .post(LOGIN_ENDPOINT.to_owned() + "/logout.xml")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Content-Length", body.chars().count())
        .body(body)
        .send();
}

fn login_campnet(
    creds: Credentials,
    client: reqwest::blocking::Client,
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
        .post(LOGIN_ENDPOINT.to_owned() + "/login.xml")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .header("Content-Length", body.chars().count())
        .body(body)
        .send();
}

static mut PROCEED_CAMPNET_ATTEMPT: bool = false;
static mut LOGOUT_CAMPNET: bool = false;

unsafe fn connect_campnet(app_handle: tauri::AppHandle) {
    let tray_handle = app_handle.tray_handle();
    let resources_resolver = app_handle.path_resolver();
    let active_icon_path = resources_resolver
        .resolve_resource("resources/icons/active.png")
        .unwrap();
    let passive_icon_path = resources_resolver
        .resolve_resource("resources/icons/passive.png")
        .unwrap();
    let file_path = app_handle
        .path_resolver()
        .app_config_dir()
        .unwrap()
        .join("credentials.json");
    let client = reqwest::blocking::Client::new();
    if PROCEED_CAMPNET_ATTEMPT {
        let campnet_status = client.head(LOGIN_ENDPOINT.to_owned()).send();
        if campnet_status.is_ok() {
            let login_status = client.head("https://www.google.com").send();
            if login_status.is_err() {
                let creds = load_creds(&file_path);
                if creds.is_ok() {
                    let res = login_campnet(creds.unwrap(), client);
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
                            PROCEED_CAMPNET_ATTEMPT = false;
                            tray_handle
                                .set_icon(tauri::Icon::File(passive_icon_path))
                                .unwrap();
                        } else if res_body.contains("exceeded") {
                            Notification::new("com.riskycase.autocampnet")
                                .title("Could not connect to Campnet!")
                                .body("Daily data limit exceeded on credentials")
                                .show()
                                .unwrap();
                            PROCEED_CAMPNET_ATTEMPT = false;
                            tray_handle
                                .set_icon(tauri::Icon::File(passive_icon_path))
                                .unwrap();
                        } else {
                            Notification::new("com.riskycase.autocampnet")
                                .title("Could not to Campnet!")
                                .body("There was an issue with the login attempt")
                                .show()
                                .unwrap();
                            PROCEED_CAMPNET_ATTEMPT = false;
                            tray_handle
                                .set_icon(tauri::Icon::File(passive_icon_path))
                                .unwrap();
                        }
                    }
                }
            }
        }
    } else if LOGOUT_CAMPNET {
        let creds = load_creds(&file_path);
        if creds.is_ok() {
            let res = logout_campnet(creds.unwrap(), client);
            if res.is_ok() {
                let res_body: String = res.unwrap().text().unwrap();
                if res_body.contains("LOGIN") {
                    Notification::new("com.riskycase.autocampnet")
                        .title("Logged out of Campnet")
                        .show()
                        .unwrap();
                }
            }
            LOGOUT_CAMPNET = false;
        }
    }

    let callback_timer = timer::Timer::new();
    let _callback_gaurd =
        callback_timer.schedule_with_delay(chrono::Duration::milliseconds(2500), move || {
            connect_campnet(app_handle.app_handle());
        });
    std::thread::sleep(std::time::Duration::from_millis(3000));
}

fn main() {
    unsafe {
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
                let file_path = app
                    .path_resolver()
                    .app_config_dir()
                    .unwrap()
                    .join("credentials.json");
                let creds = load_creds(&file_path);
                if creds.is_ok() {
                    PROCEED_CAMPNET_ATTEMPT = true;
                } else {
                    app.get_window("main").unwrap().show().unwrap();
                }
                app.listen_global("save", move |event: tauri::Event| {
                    let creds: Credentials =
                        serde_json::from_str(event.payload().unwrap()).unwrap();
                    save_creds(creds, &file_path);
                    PROCEED_CAMPNET_ATTEMPT = true;
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
                connect_campnet(app.handle());
                std::fs::create_dir_all(app.path_resolver().app_config_dir().unwrap()).unwrap();
                Ok(())
            })
            .system_tray(system_tray)
            .on_system_tray_event(|app: &tauri::AppHandle, event| match event {
                tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                    "quit" => {
                        std::process::exit(0);
                    }
                    "show" => {
                        let window: tauri::Window = app.get_window("main").unwrap();
                        let file_path = app
                            .path_resolver()
                            .app_config_dir()
                            .unwrap()
                            .join("credentials.json");
                        let creds = load_creds(&file_path);
                        if creds.is_ok() {
                            window.emit("credentials", creds.unwrap()).unwrap();
                        } else {
                            window
                                .emit(
                                    "credentials",
                                    Credentials {
                                        username: "".into(),
                                        password: "".into(),
                                    },
                                )
                                .unwrap();
                        }
                        window.show().unwrap();
                        window.unminimize().unwrap();
                        window.set_focus().unwrap();
                    }
                    "logout" => {
                        LOGOUT_CAMPNET = true;
                        PROCEED_CAMPNET_ATTEMPT = false;
                        app.tray_handle()
                            .set_icon(tauri::Icon::File(
                                app.path_resolver()
                                    .resolve_resource("resources/icons/passive.png")
                                    .unwrap(),
                            ))
                            .unwrap();
                    }
                    "reconnect" => {
                        let file_path = app
                            .path_resolver()
                            .app_config_dir()
                            .unwrap()
                            .join("credentials.json");
                        let creds = load_creds(&file_path);
                        if creds.is_ok() {
                            if PROCEED_CAMPNET_ATTEMPT {
                                connect_campnet(app.app_handle());
                            }
                            PROCEED_CAMPNET_ATTEMPT = true;
                        } else {
                            let window: tauri::Window = app.get_window("main").unwrap();
                            window.show().unwrap();
                        }
                    }
                    "delete" => {
                        let window: tauri::Window = app.get_window("main").unwrap();
                        window
                            .emit(
                                "credentials",
                                Credentials {
                                    username: "".into(),
                                    password: "".into(),
                                },
                            )
                            .unwrap();
                        window.show().unwrap();
                        let file_path = app
                            .path_resolver()
                            .app_config_dir()
                            .unwrap()
                            .join("credentials.json");
                        let creds = load_creds(&file_path);
                        if creds.is_ok() {
                            std::fs::remove_file(&file_path).unwrap();
                        }
                        PROCEED_CAMPNET_ATTEMPT = false;
                    }
                    _ => {}
                },
                tauri::SystemTrayEvent::LeftClick {
                    tray_id: _,
                    position: _,
                    size: _,
                    ..
                } => {
                    let window: tauri::Window = app.get_window("main").unwrap();
                        let file_path = app
                            .path_resolver()
                            .app_config_dir()
                            .unwrap()
                            .join("credentials.json");
                        let creds = load_creds(&file_path);
                    if creds.is_ok() {
                        window.emit("credentials", creds.unwrap()).unwrap();
                    } else {
                        window
                            .emit(
                                "credentials",
                                Credentials {
                                    username: "".into(),
                                    password: "".into(),
                                },
                            )
                            .unwrap();
                    }
                    window.show().unwrap();
                    window.unminimize().unwrap();
                    window.set_focus().unwrap();
                }
                _ => {}
            })
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    }
}
