use tauri::Manager;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Credentials {
  username: String,
  password: String
}

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      app.listen_global("save", |event| {
        let creds: Credentials = serde_json::from_str(event.payload().unwrap()).unwrap();
        println!("Got credentials with username: {} and password: {}", creds.username, creds.password);
      });
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
