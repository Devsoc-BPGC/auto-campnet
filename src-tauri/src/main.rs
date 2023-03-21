// Do not show a console window on Windows
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use auto_launch::{AutoLaunch, AutoLaunchBuilder, Error};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::env::current_exe;
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

#[derive(Clone, PartialEq, Copy)]
enum NotificationState {
    None,
    Used50,
    Used90,
    Used100,
}

#[derive(Clone)]
struct AppState {
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

pub struct AutoLaunchManager(AutoLaunch);

impl AutoLaunchManager {
    pub fn enable(&self) -> Result<(), Error> {
        self.0.enable()
    }

    pub fn disable(&self) -> Result<(), Error> {
        self.0.disable()
    }

    pub fn is_enabled(&self) -> Result<bool, Error> {
        self.0.is_enabled()
    }
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
        let app_state = app.state::<Arc<Mutex<AppState>>>();
        app_state.lock().unwrap().login_guard = Option::None;
        let tray_handle = app.tray_handle();
        let resources_resolver = app.path_resolver();
        let active_icon_path = resources_resolver
            .resolve_resource("resources/icons/active.png")
            .unwrap();
        let used_50_icon_path = resources_resolver
            .resolve_resource("resources/icons/used_50.png")
            .unwrap();
        let used_90_icon_path = resources_resolver
            .resolve_resource("resources/icons/used_90.png")
            .unwrap();
        let inactive_icon_path = resources_resolver
            .resolve_resource("resources/icons/inactive.png")
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
                        let current_notification_state =
                            app_state.lock().unwrap().last_notification_state;
                        if current_notification_state == NotificationState::None {
                            tray_handle
                                .set_icon(tauri::Icon::File(active_icon_path))
                                .unwrap();
                        } else if current_notification_state == NotificationState::Used50 {
                            tray_handle
                                .set_icon(tauri::Icon::File(used_50_icon_path))
                                .unwrap();
                        } else if current_notification_state == NotificationState::Used90 {
                            tray_handle
                                .set_icon(tauri::Icon::File(used_90_icon_path))
                                .unwrap();
                        }
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
                            .set_icon(tauri::Icon::File(inactive_icon_path))
                            .unwrap();
                    } else if res_body.contains("exceeded") {
                        Notification::new("com.riskycase.autocampnet")
                            .title("Could not connect to Campnet!")
                            .body("Daily data limit exceeded on credentials")
                            .show()
                            .unwrap();
                        tray_handle
                            .set_icon(tauri::Icon::File(inactive_icon_path))
                            .unwrap();
                    } else {
                        Notification::new("com.riskycase.autocampnet")
                            .title("Could not to Campnet!")
                            .body("There was an issue with the login attempt")
                            .show()
                            .unwrap();
                        tray_handle
                            .set_icon(tauri::Icon::File(inactive_icon_path))
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
        let app_state = app.state::<Arc<Mutex<AppState>>>();
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
    let app_state = app.state::<Arc<Mutex<AppState>>>();
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

fn get_csrf(app: tauri::AppHandle) -> Result<(), ()> {
    let client = reqwest::blocking::Client::new();
    let app_state = app.state::<Arc<Mutex<AppState>>>();
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
        let matches = regex.captures(body.as_str());
        if matches.is_some() {
            app_state.lock().unwrap().csrf = matches
                .unwrap()
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
            Err(())
        }
    } else {
        Err(())
    }
}

fn get_remaining_data(app: tauri::AppHandle, initial_run: bool) {
    if !initial_run {
        let app_state = app.state::<Arc<Mutex<AppState>>>();
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
                        let body_text = data_result.unwrap().text().unwrap();
                        let dom =
                            tl::parse(body_text.as_str(), tl::ParserOptions::default()).unwrap();
                        let parser = dom.parser();
                        let element = dom
                            .get_element_by_id("content3")
                            .expect("")
                            .get(parser)
                            .unwrap();
                        let table_text = element.inner_html(parser).to_string();
                        let sub_dom =
                            tl::parse(table_text.as_str(), tl::ParserOptions::default()).unwrap();
                        let sub_parser = sub_dom.parser();
                        let mut data_vector: Vec<f32> = Vec::new();
                        let mut unit_vector: Vec<String> = Vec::new();
                        let datas = sub_dom.query_selector("td.tabletext").unwrap();
                        datas.for_each(|data| {
                            data_vector.push(
                                data.get(sub_parser)
                                    .unwrap()
                                    .inner_text(sub_parser)
                                    .trim()
                                    .replace("&nbsp;", "")
                                    .parse::<f32>()
                                    .unwrap(),
                            );
                            unit_vector.push(
                                data.get(sub_parser)
                                    .unwrap()
                                    .children()
                                    .unwrap()
                                    .all(sub_parser)
                                    .get(1)
                                    .unwrap()
                                    .outer_html(sub_parser)
                                    .to_string()
                                    .split(".")
                                    .nth(1)
                                    .unwrap()
                                    .split("\"")
                                    .nth(0)
                                    .unwrap()
                                    .to_string(),
                            );
                        });
                        let traffic = TrafficStats {
                            total: data_vector[6],
                            last: data_vector[7],
                            current: data_vector[8],
                            used: data_vector[9],
                            remaining: data_vector[10],
                        };
                        app_state.lock().unwrap().traffic = traffic.clone();
                        let data_usage = traffic.used / traffic.total;
                        let current_notification_state = if data_usage < 0.5 {
                            NotificationState::None
                        } else if data_usage < 0.9 {
                            NotificationState::Used50
                        } else if data_usage < 1.0 {
                            NotificationState::Used90
                        } else {
                            NotificationState::Used100
                        };
                        let traffic_units = TrafficUnits {
                            total: unit_vector[6].to_string(),
                            last: unit_vector[7].to_string(),
                            current: unit_vector[8].to_string(),
                            used: unit_vector[9].to_string(),
                            remaining: unit_vector[10].to_string(),
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
        let app_state = app.state::<Arc<Mutex<AppState>>>();
        let callback_timer = timer::Timer::new();
        let callback_gaurd =
            callback_timer.schedule_with_delay(chrono::Duration::zero(), move || {
                get_remaining_data(app_handle_next.app_handle(), false);
            });
        app_state.lock().unwrap().traffic_guard = Option::Some(callback_gaurd.to_owned());
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

#[tauri::command]
fn credential_check(
    username: String,
    password: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let app_state = app.state::<Arc<Mutex<AppState>>>();
    let client = reqwest::blocking::Client::new();
    let campnet_status = client
        .head(app_state.lock().unwrap().login_endpoint.to_owned())
        .send();
    if campnet_status.is_ok() {
        let body: String = format!(
            "mode=193&username={}&a={}&producttype=1",
            username,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );
        let initial_res = client
            .post(app_state.lock().unwrap().login_endpoint.to_owned() + "/logout.xml")
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Content-Length", body.chars().count())
            .body(body)
            .send();
        if initial_res.is_ok() {
            let res = login_campnet(
                client,
                Credentials { username, password },
                app_state.lock().unwrap().login_endpoint.to_string(),
            );
            if res.is_ok() {
                let res_body: String = res.unwrap().text().unwrap();
                if res_body.contains("LIVE") || res_body.contains("exceeded") {
                    Ok(())
                } else if res_body.contains("failed") {
                    Err("INVALIDCRED".to_string())
                } else {
                    Err("UNKNOWN".to_string())
                }
            } else {
                Err("UNKNOWN".to_string())
            }
        } else {
            Err("UNKNOWN".to_string())
        }
    } else {
        Err("NOSOPHOS".to_string())
    }
}

fn auto_launch_check(app: tauri::AppHandle) {
    let window: tauri::Window = app.get_window("main").unwrap();
    window
        .emit(
            "autolaunch",
            app.state::<AutoLaunchManager>().is_enabled().unwrap(),
        )
        .unwrap();
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
            app.manage(Arc::new(Mutex::new(AppState {
                login_endpoint: "https://campnet.bits-goa.ac.in:8090".to_string(),
                credentials: Credentials {
                    username: "".to_string(),
                    password: "".to_string(),
                },
                login_guard: Option::None,
                portal_endpoint: "https://campnet.bits-goa.ac.in:8093".to_string(),
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
                let app_state = app_handle_save.state::<Arc<Mutex<AppState>>>();
                app_state.lock().unwrap().credentials = creds.clone();
                save_creds(creds, &file_path);
                let app_handle_thread = app_handle_save.app_handle();
                std::thread::spawn(move || {
                    connect_campnet(app_handle_thread.app_handle(), false);
                    get_remaining_data(app_handle_thread.app_handle(), false);
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
            let mut auto_launch_builder = AutoLaunchBuilder::new();
            auto_launch_builder.set_app_name(&app.package_info().name);
            let currnet_exe = current_exe();
            #[cfg(windows)]
            auto_launch_builder.set_app_path(&currnet_exe.unwrap().display().to_string());
            #[cfg(target_os = "macos")]
            {
                // on macOS, current_exe gives path to /Applications/Example.app/MacOS/Example
                // but this results in seeing a Unix Executable in macOS login items
                // It must be: /Applications/Example.app
                // If it didn't find exactly a single occurance of .app, it will default to
                // exe path to not break it.
                let exe_path = currnet_exe.unwrap().canonicalize()?.display().to_string();
                let parts: Vec<&str> = exe_path.split(".app/").collect();
                let app_path = if parts.len() == 2 {
                    format!("{}.app", parts.get(0).unwrap().to_string())
                } else {
                    exe_path
                };
                info!("auto_start path {}", &app_path);
                auto_launch_builder.set_app_path(&app_path);
            }
            #[cfg(target_os = "linux")]
            if let Some(appimage) = app
                .env()
                .appimage
                .and_then(|p| p.to_str().map(|s| s.to_string()))
            {
                auto_launch_builder.set_app_path(&appimage);
            } else {
                auto_launch_builder.set_app_path(&currnet_exe.unwrap().display().to_string());
            }

            app.manage(AutoLaunchManager(
                auto_launch_builder.build().map_err(|e| e.to_string())?,
            ));

            let app_handle_launch = app.app_handle();
            app.listen_global("autolaunch", move |event: tauri::Event| {
                if event.payload().unwrap().parse::<bool>().unwrap() {
                    app_handle_launch
                        .state::<AutoLaunchManager>()
                        .enable()
                        .unwrap();
                } else {
                    app_handle_launch
                        .state::<AutoLaunchManager>()
                        .disable()
                        .unwrap();
                }
                auto_launch_check(app_handle_launch.app_handle());
            });

            std::fs::create_dir_all(app.path_resolver().app_config_dir().unwrap()).unwrap();
            let app_state: State<Arc<Mutex<AppState>>> = app.state::<Arc<Mutex<AppState>>>();
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
                auto_launch_check(app.app_handle());
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
                    let app_state = app.state::<Arc<Mutex<AppState>>>();
                    auto_launch_check(app.app_handle());
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
                    let app_state = app_handle_logout.state::<Arc<Mutex<AppState>>>();
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
                                    .resolve_resource("resources/icons/inactive.png")
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
                    let app_state = app.state::<Arc<Mutex<AppState>>>();
                    let creds = app_state.lock().unwrap().credentials.to_owned();
                    app_state.lock().unwrap().login_guard = Option::None;
                    app_state.lock().unwrap().traffic_guard = Option::None;
                    if (creds.username.len() == 0) | (creds.password.len() == 0) {
                        let window: tauri::Window = app.get_window("main").unwrap();
                        window.show().unwrap();
                    } else {
                        connect_campnet(app.app_handle(), false);
                        get_remaining_data(app.app_handle(), false);
                    }
                }
                "delete" => {
                    let file_path = app
                        .path_resolver()
                        .app_config_dir()
                        .unwrap()
                        .join("credentials.json");
                    std::fs::remove_file(&file_path).unwrap();
                    let app_state = app.state::<Arc<Mutex<AppState>>>();
                    app_state.lock().unwrap().login_guard = Option::None;
                    app_state.lock().unwrap().traffic_guard = Option::None;
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
                    auto_launch_check(app.app_handle());
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
                let app_state = app.state::<Arc<Mutex<AppState>>>();
                auto_launch_check(app.app_handle());
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
        .invoke_handler(tauri::generate_handler![credential_check])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
