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
  writeln!(&mut file, "{}", serde_json::to_string(&creds).unwrap()).unwrap();
}


fn main() {
  tauri::Builder::default()
  .setup(|app: &mut tauri::App| {
    let save_dir = tauri::api::path::app_dir(&app.config()).unwrap();
    let save_file = save_dir.join("credentials.json");
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
