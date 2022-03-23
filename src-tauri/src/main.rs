use tauri::{Manager, api::{path, file}};
use serde::{Deserialize, Serialize};
use std::io::Write;
extern crate timer;
extern crate chrono;

#[derive(Serialize, Deserialize)]
struct Credentials {
  username: String,
  password: String
}

fn save_creds(creds: Credentials, save_file: &std::path::Path){
  let mut file = std::fs::File::create(&save_file).unwrap();
  write!(&mut file, "{}", serde_json::to_string(&creds).unwrap()).unwrap();
}

fn load_creds(save_file: &std::path::Path) -> Result<Credentials, String> {
  let creds_string = file::read_string(save_file);
  if creds_string.is_ok() {
    let creds: Credentials = serde_json::from_str(&creds_string.unwrap()).unwrap();
    return Ok(creds);
  }
  else {
    return Err("Credentials not saved".to_string());
  }
}

static mut proceed_campnet_attempt: bool = false;

unsafe fn connect_campnet(file_path: &std::path::PathBuf) {
  
  if proceed_campnet_attempt {
    let campnet_status = reqwest::blocking::get("https://campnet.bits-goa.ac.in:8090/");
    if campnet_status.is_ok() {
      let login_status = reqwest::blocking::get("https://www.google.com");
      if login_status.is_err() {
        let helper_file = file_path.parent().unwrap().join("credentials.json");
        let creds = load_creds(&helper_file);
        if creds.is_ok() {
          let creds = creds.unwrap();
          let body: String = format!("mode=191&username={}&password={}&a={}&producttype=1", creds.username, creds.password, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
          let client = reqwest::blocking::Client::new();
          let res = client.post("https://campnet.bits-goa.ac.in:8090/login.xml")
              .header("Content-Type", "application/x-www-form-urlencoded")
              .header("Content-Length", body.chars().count())
              .body(body)
              .send();
          if res.is_ok() {
            let res_body: String = res.unwrap().text().unwrap();
            if res_body.contains("LIVE") {
              tauri::api::notification::Notification::new("com.riskycase.autocampnet")
                .title("Connected to Campnet!")
                .body("Logged in successfully to BPGC network")
                .show();
              println!("Conn succ");
            }
            else {
              tauri::api::notification::Notification::new("com.riskycase.autocampnet")
                .title("Could not to Campnet!")
                .body("There was an issue with the login attempt")
                .show();
              println!("Conn issue");
            }
          }
        }
      }
      else {
        println!("loggeed in already");
      }
    }
    else {
      println!("Not campnet");
    }
  }

  let callback_timer = timer::Timer::new();
  let callback_path = file_path.parent().unwrap().join("credentials.json");
  let _callback_gaurd = callback_timer.schedule_with_delay(chrono::Duration::milliseconds(2500), move || {
    connect_campnet(&callback_path);
  });
  std::thread::sleep(std::time::Duration::from_millis(3000));
}

fn main() {
  tauri::Builder::default()
  .setup(|app: &mut tauri::App| unsafe {
    let save_dir = path::app_dir(&app.config()).unwrap();
    let file_creds = load_creds(&(save_dir.join("credentials.json")));
    if file_creds.is_ok() {
      let _creds = file_creds.unwrap();
      proceed_campnet_attempt = true;
    }
    else {
      println!("Credentials absent");
    }
    let write_save_file = save_dir.join("credentials.json");
    app.listen_global("save", move |event: tauri::Event| {
      let creds: Credentials = serde_json::from_str(event.payload().unwrap()).unwrap();
      save_creds(creds, &write_save_file);
      proceed_campnet_attempt = true;
    });
    let read_save_file = save_dir.join("credentials.json");
    connect_campnet(&read_save_file);
    std::fs::create_dir_all(save_dir).unwrap();
    Ok(())
  })
  .run(tauri::generate_context!())
  .expect("error while running tauri application");
}
