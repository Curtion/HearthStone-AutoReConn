use std::process::Command;

pub fn get_hs_path() -> String {
  let output = Command::new("wmic")
    .args(&[
      "Process",
      "where",
      // "name='Hearthstone.exe'",
      "name='QQ.exe'",
      "get",
      "executablepath",
    ])
    .output()
    .expect("failed to execute process");
  let path = String::from_utf8_lossy(&output.stdout).to_string();
  let path: Vec<&str> = path.split_whitespace().collect();
  path[1].to_string()
}
