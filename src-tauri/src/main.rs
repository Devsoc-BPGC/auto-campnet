use tauri::Manager;
use serde::{Deserialize, Serialize};
use std::io::Write;

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
  let creds_string = tauri::api::file::read_string(save_file);
  if creds_string.is_ok() {
    let creds: Credentials = serde_json::from_str(&creds_string.unwrap()).unwrap();
    return Ok(creds);
  }
  else {
    return Err("Credentials not saved".to_string());
  }
}

fn main() {
  tauri::Builder::default()
  .setup(|app: &mut tauri::App| {
    let save_dir = tauri::api::path::app_dir(&app.config()).unwrap();
    let save_file = save_dir.join("credentials.json");
    let file_creds = load_creds(&(save_dir.join("credentials.json")));
    if file_creds.is_ok() {
      println!("Credentials present");
      let creds = file_creds.unwrap();
      println!("Username: {}", creds.username);
      println!("Password: {}", creds.password);
    }
    else {
      println!("Credentials absent");
    }
    std::fs::create_dir_all(save_dir).unwrap();
    app.listen_global("save", move |event: tauri::Event| {
      let creds: Credentials = serde_json::from_str(event.payload().unwrap()).unwrap();
      save_creds(creds, &save_file);
    });
    Ok(())
  })
  .run(tauri::generate_context!())
  .expect("error while running tauri application");
}
