// Do not show a console window on Windows
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::sync::{Arc, Mutex};
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

#[derive(Serialize, Deserialize, Clone)]
struct TrafficStats {
    total: f32,
    last: f32,
    current: f32,
    used: f32,
    remaining: f32,
}

#[derive(Serialize, Deserialize, Clone)]
struct TrafficUnits {
    total: String,
    last: String,
    current: String,
    used: String,
    remaining: String,
}

#[derive(Clone, PartialEq)]
enum NotificationState {
    None,
    Used50,
    Used90,
}

#[derive(Clone)]
struct ConnectState {
    login_endpoint: String,
    credentials: Credentials,
    login_guard: Option<timer::Guard>,
    portal_endpoint: String,
    cookie: String,
    csrf: String,
    traffic: TrafficStats,
    traffic_units: TrafficUnits,
    traffic_guard: Option<timer::Guard>,
    last_notification_state: NotificationState,
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

fn connect_campnet(app: tauri::AppHandle, initial_run: bool) {
    if !initial_run {
        let app_state = app.state::<Arc<Mutex<ConnectState>>>();
        app_state.lock().unwrap().login_guard = Option::None;
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
                        let app_handle_next = app.app_handle();
                        let callback_timer = timer::Timer::new();
                        let callback_gaurd = callback_timer.schedule_with_delay(
                            chrono::Duration::milliseconds(2500),
                            move || {
                                connect_campnet(app_handle_next.app_handle(), false);
                            },
                        );
                        app_state.lock().unwrap().login_guard =
                            Option::Some(callback_gaurd.to_owned());
                        std::thread::sleep(std::time::Duration::from_secs(3));
                    } else if res_body.contains("failed") {
                        Notification::new("com.riskycase.autocampnet")
                            .title("Could not connect to Campnet!")
                            .body("Incorrect credentials were provided")
                            .show()
                            .unwrap();
                        tray_handle
                            .set_icon(tauri::Icon::File(passive_icon_path))
                            .unwrap();
                    } else if res_body.contains("exceeded") {
                        Notification::new("com.riskycase.autocampnet")
                            .title("Could not connect to Campnet!")
                            .body("Daily data limit exceeded on credentials")
                            .show()
                            .unwrap();
                        tray_handle
                            .set_icon(tauri::Icon::File(passive_icon_path))
                            .unwrap();
                    } else {
                        Notification::new("com.riskycase.autocampnet")
                            .title("Could not to Campnet!")
                            .body("There was an issue with the login attempt")
                            .show()
                            .unwrap();
                        tray_handle
                            .set_icon(tauri::Icon::File(passive_icon_path))
                            .unwrap();
                    }
                }
            } else {
                let app_handle_next = app.app_handle();
                let callback_timer = timer::Timer::new();
                let callback_gaurd = callback_timer.schedule_with_delay(
                    chrono::Duration::milliseconds(2500),
                    move || {
                        connect_campnet(app_handle_next.app_handle(), false);
                    },
                );
                app_state.lock().unwrap().login_guard = Option::Some(callback_gaurd.to_owned());
                tray_handle
                    .set_icon(tauri::Icon::File(active_icon_path))
                    .unwrap();
                std::thread::sleep(std::time::Duration::from_secs(3));
            }
        }
    } else {
        let app_handle_next = app.app_handle();
        let app_state = app.state::<Arc<Mutex<ConnectState>>>();
        let callback_timer = timer::Timer::new();
        let callback_gaurd =
            callback_timer.schedule_with_delay(chrono::Duration::zero(), move || {
                connect_campnet(app_handle_next.app_handle(), false);
            });
        app_state.lock().unwrap().login_guard = Option::Some(callback_gaurd.to_owned());
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn get_cookie(app: tauri::AppHandle) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let app_state = app.state::<Arc<Mutex<ConnectState>>>();
    let credentials = app_state.lock().unwrap().credentials.clone();
    let body: String = format!(
        "mode=451&json=%7B%22username%22%3A%22{}%22%2C%22password%22%3A%22{}%22%2C%22languageid%22%3A%221%22%2C%22browser%22%3A%22Chrome_109%22%7D&t={}",
        credentials.username,
        credentials.password,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    );
    let response = client
        .post(app_state.lock().unwrap().portal_endpoint.to_owned() + "/userportal/Controller")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send();
    if response.is_ok() {
        app_state.lock().unwrap().cookie = response
            .unwrap()
            .headers()
            .get(reqwest::header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap()
            .split(";")
            .into_iter()
            .nth(0)
            .unwrap()
            .to_string();
        Ok(())
    } else {
        Err(response.err().unwrap())
    }
}

fn get_csrf(app: tauri::AppHandle) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let app_state = app.state::<Arc<Mutex<ConnectState>>>();
    let cookie = app_state.lock().unwrap().cookie.to_string();
    let response = client
        .get(
            app_state.lock().unwrap().portal_endpoint.to_string()
                + "/userportal/webpages/myaccount/index.jsp",
        )
        .header(reqwest::header::COOKIE, cookie.to_string())
        .header(
            reqwest::header::USER_AGENT,
            format!("AutoCampnetRuntime/{}", app.package_info().version),
        )
        .send();
    if response.is_ok() {
        let regex = Regex::new(r"k3n = '(.+)'").unwrap();
        let body = response.unwrap().text().unwrap();
        let matches = regex.captures(body.as_str()).unwrap();
        app_state.lock().unwrap().csrf = matches
            .get(0)
            .unwrap()
            .as_str()
            .split("'")
            .into_iter()
            .nth(1)
            .unwrap()
            .to_string();
        Ok(())
    } else {
        Err(response.err().unwrap())
    }
}

fn get_remaining_data(app: tauri::AppHandle, initial_run: bool) {
    if !initial_run {
        let app_state = app.state::<Arc<Mutex<ConnectState>>>();
        app_state.lock().unwrap().traffic_guard = Option::None;
        let client = reqwest::blocking::Client::new();
        let campnet_status = client
            .head(app_state.lock().unwrap().login_endpoint.to_owned())
            .send();
        if campnet_status.is_ok() {
            let cookie_result = get_cookie(app.app_handle());
            if cookie_result.is_ok() {
                let csrf_result = get_csrf(app.app_handle());
                if csrf_result.is_ok() {
                    let cookie = app_state.lock().unwrap().cookie.to_string();
                    let csrf = app_state.lock().unwrap().csrf.to_string();
                    let portal_endpoint = app_state.lock().unwrap().portal_endpoint.to_string();
                    let data_result = client
                        .get(
                            portal_endpoint.to_string()
                                + "/userportal/webpages/myaccount/AccountStatus.jsp",
                        )
                        .query(&[
                            ("popup", "0"),
                            (
                                "t",
                                format!(
                                    "{}",
                                    std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_millis()
                                )
                                .as_str(),
                            ),
                        ])
                        .header("X-CSRF-Token", csrf)
                        .header(reqwest::header::COOKIE, cookie)
                        .header(
                            reqwest::header::USER_AGENT,
                            format!("AutoCampnetRuntime/{}", app.package_info().version),
                        )
                        .header(
                            reqwest::header::REFERER,
                            portal_endpoint.to_string()
                                + "/userportal/webpages/myaccount/login.jsp",
                        )
                        .send();
                    if data_result.is_ok() {
                        let regex_traffic = Regex::new(r">\s+(\d+\.?\d*)").unwrap();
                        let regex_traffic_units =
                            Regex::new(r">\s+\d+\.?\d*&nbsp;<label id='Language\.(\w+)").unwrap();
                        let body = data_result
                            .unwrap()
                            .text()
                            .unwrap()
                            .split("Language.CycleDataTrasfer")
                            .into_iter()
                            .nth(1)
                            .unwrap()
                            .to_string();
                        let mut matches = regex_traffic
                            .find_iter(body.split("</table>").into_iter().nth(0).unwrap())
                            .into_iter()
                            .map(|data| {
                                data.as_str()
                                    .replace(">", "")
                                    .trim()
                                    .to_string()
                                    .parse::<f32>()
                                    .unwrap()
                            });
                        let traffic = TrafficStats {
                            total: matches.next().unwrap(),
                            last: matches.next().unwrap(),
                            current: matches.next().unwrap(),
                            used: matches.next().unwrap(),
                            remaining: matches.next().unwrap(),
                        };
                        app_state.lock().unwrap().traffic = traffic.clone();
                        let data_usage = traffic.used / traffic.total;
                        let current_notification_state = if data_usage < 0.5 {
                            NotificationState::None
                        } else if data_usage < 0.9 {
                            NotificationState::Used50
                        } else {
                            NotificationState::Used90
                        };
                        let mut matches_unit = regex_traffic_units
                            .find_iter(body.split("</table>").into_iter().nth(0).unwrap())
                            .into_iter()
                            .map(|data| {
                                data.as_str()
                                    .replace(">", "")
                                    .trim()
                                    .to_string()
                                    .split("Language.")
                                    .into_iter()
                                    .nth(1)
                                    .unwrap()
                                    .to_string()
                            });
                        let traffic_units = TrafficUnits {
                            total: matches_unit.next().unwrap().to_string(),
                            last: matches_unit.next().unwrap().to_string(),
                            current: matches_unit.next().unwrap().to_string(),
                            used: matches_unit.next().unwrap().to_string(),
                            remaining: matches_unit.next().unwrap().to_string(),
                        };
                        app_state.lock().unwrap().traffic_units = traffic_units.clone();
                        if app_state.lock().unwrap().last_notification_state
                            != current_notification_state
                        {
                            if current_notification_state == NotificationState::Used50 {
                                Notification::new("com.riskycase.autocampnet")
                                    .title("50% data warning!")
                                    .body("Consider slowing down")
                                    .show()
                                    .unwrap();
                            } else if current_notification_state == NotificationState::Used90 {
                                Notification::new("com.riskycase.autocampnet")
                                    .title("90% data warning!")
                                    .body("Tread the interwebs slowly")
                                    .show()
                                    .unwrap();
                            }
                            app_state.lock().unwrap().last_notification_state =
                                current_notification_state
                        }
                        app.get_window("main")
                            .unwrap()
                            .emit("traffic", traffic.clone())
                            .unwrap();
                        app.get_window("main")
                            .unwrap()
                            .emit("traffic_units", traffic_units.clone())
                            .unwrap();
                    }
                }
            }
        }
        let app_handle_next = app.app_handle();
        let callback_timer = timer::Timer::new();
        let callback_gaurd =
            callback_timer.schedule_with_delay(chrono::Duration::seconds(45), move || {
                get_remaining_data(app_handle_next.app_handle(), false);
            });
        app_state.lock().unwrap().traffic_guard = Option::Some(callback_gaurd.to_owned());
        std::thread::sleep(std::time::Duration::from_secs(55));
    } else {
        let app_handle_next = app.app_handle();
        let app_state = app.state::<Arc<Mutex<ConnectState>>>();
        let callback_timer = timer::Timer::new();
        let callback_gaurd =
            callback_timer.schedule_with_delay(chrono::Duration::zero(), move || {
                get_remaining_data(app_handle_next.app_handle(), false);
            });
        app_state.lock().unwrap().traffic_guard = Option::Some(callback_gaurd.to_owned());
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
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
            app.manage(Arc::new(Mutex::new(ConnectState {
                login_endpoint: String::new(),
                credentials: Credentials {
                    username: "".to_string(),
                    password: "".to_string(),
                },
                login_guard: Option::None,
                portal_endpoint: "".to_string(),
                cookie: "".to_string(),
                csrf: "".to_string(),
                traffic: TrafficStats {
                    total: 0.0,
                    last: 0.0,
                    current: 0.0,
                    used: 0.0,
                    remaining: 0.0,
                },
                traffic_units: TrafficUnits {
                    total: "".to_string(),
                    last: "".to_string(),
                    current: "".to_string(),
                    used: "".to_string(),
                    remaining: "".to_string(),
                },
                traffic_guard: Option::None,
                last_notification_state: NotificationState::None,
            })));
            let file_path = app
                .path_resolver()
                .app_config_dir()
                .unwrap()
                .join("credentials.json");
            let creds = load_creds(&file_path);
            let app_handle_save = app.app_handle();
            app.listen_global("save", move |event: tauri::Event| {
                let creds: Credentials = serde_json::from_str(event.payload().unwrap()).unwrap();
                let app_state = app_handle_save.state::<Arc<Mutex<ConnectState>>>();
                app_state.lock().unwrap().credentials = creds.clone();
                save_creds(creds, &file_path);
                let app_handle_thread = app_handle_save.app_handle();
                std::thread::spawn(move || {
                    connect_campnet(app_handle_thread.app_handle(), false);
                });
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
            let app_state: State<Arc<Mutex<ConnectState>>> =
                app.state::<Arc<Mutex<ConnectState>>>();
            if creds.is_ok() {
                app_state.lock().unwrap().login_endpoint =
                    String::from("https://campnet.bits-goa.ac.in:8090");
                app_state.lock().unwrap().portal_endpoint =
                    String::from("https://campnet.bits-goa.ac.in:8093");
                app_state.lock().unwrap().credentials = creds.unwrap();
                connect_campnet(app.app_handle(), true);
                get_remaining_data(app.app_handle(), true);
            } else {
                app.get_window("main").unwrap().show().unwrap();
            }
            Ok(())
        })
        .system_tray(system_tray)
        .on_system_tray_event(|app: &tauri::AppHandle, event| match event {
            tauri::SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                "show" => {
                    let app_state = app.state::<Arc<Mutex<ConnectState>>>();
                    let window: tauri::Window = app.get_window("main").unwrap();
                    window
                        .emit("credentials", app_state.lock().unwrap().credentials.clone())
                        .unwrap();
                    window
                        .emit("traffic", app_state.lock().unwrap().traffic.clone())
                        .unwrap();
                    window
                        .emit(
                            "traffic_units",
                            app_state.lock().unwrap().traffic_units.clone(),
                        )
                        .unwrap();
                    window.show().unwrap();
                    window.unminimize().unwrap();
                    window.set_focus().unwrap();
                }
                "logout" => {
                    let app_handle_logout = app.app_handle();
                    let client = reqwest::blocking::Client::new();
                    let app_state = app_handle_logout.state::<Arc<Mutex<ConnectState>>>();
                    app_state.lock().unwrap().login_guard = Option::None;
                    let body: String = format!(
                        "mode=193&username={}&a={}&producttype=1",
                        app_state.lock().unwrap().credentials.username,
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_millis()
                    );
                    let res = client
                        .post(app_state.lock().unwrap().login_endpoint.to_owned() + "/logout.xml")
                        .header("Content-Type", "application/x-www-form-urlencoded")
                        .header("Content-Length", body.chars().count())
                        .body(body)
                        .send();
                    if res.is_ok() {
                        app.tray_handle()
                            .set_icon(tauri::Icon::File(
                                app.path_resolver()
                                    .resolve_resource("resources/icons/passive.png")
                                    .unwrap(),
                            ))
                            .unwrap();
                        Notification::new("com.riskycase.autocampnet")
                            .title("Logged out of campnet!")
                            .show()
                            .unwrap();
                    } else {
                        Notification::new("com.riskycase.autocampnet")
                            .title("Unable to logout of campnet!")
                            .show()
                            .unwrap();
                    }
                }
                "reconnect" => {
                    let app_state = app.state::<Arc<Mutex<ConnectState>>>();
                    let creds = app_state.lock().unwrap().credentials.to_owned();
                    app_state.lock().unwrap().login_guard = Option::None;
                    if (creds.username.len() == 0) | (creds.password.len() == 0) {
                        let window: tauri::Window = app.get_window("main").unwrap();
                        window.show().unwrap();
                    } else {
                        connect_campnet(app.app_handle(), false);
                    }
                }
                "delete" => {
                    let file_path = app
                        .path_resolver()
                        .app_config_dir()
                        .unwrap()
                        .join("credentials.json");
                    std::fs::remove_file(&file_path).unwrap();
                    let app_state = app.state::<Arc<Mutex<ConnectState>>>();
                    app_state.lock().unwrap().login_guard = Option::None;
                    app_state.lock().unwrap().credentials = Credentials {
                        username: "".to_owned(),
                        password: "".to_owned(),
                    };
                    app_state.lock().unwrap().traffic = TrafficStats {
                        total: 0.0,
                        last: 0.0,
                        current: 0.0,
                        used: 0.0,
                        remaining: 0.0,
                    };
                    app_state.lock().unwrap().traffic_units = TrafficUnits {
                        total: "".to_string(),
                        last: "".to_string(),
                        current: "".to_string(),
                        used: "".to_string(),
                        remaining: "".to_string(),
                    };
                    let window: tauri::Window = app.get_window("main").unwrap();
                    window
                        .emit("credentials", app_state.lock().unwrap().credentials.clone())
                        .unwrap();
                    window
                        .emit("traffic", app_state.lock().unwrap().traffic.clone())
                        .unwrap();
                    window
                        .emit(
                            "traffic_units",
                            app_state.lock().unwrap().traffic_units.clone(),
                        )
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
                let app_state = app.state::<Arc<Mutex<ConnectState>>>();
                let window: tauri::Window = app.get_window("main").unwrap();
                window
                    .emit("credentials", app_state.lock().unwrap().credentials.clone())
                    .unwrap();
                window
                    .emit("traffic", app_state.lock().unwrap().traffic.clone())
                    .unwrap();
                window
                    .emit(
                        "traffic_units",
                        app_state.lock().unwrap().traffic_units.clone(),
                    )
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
