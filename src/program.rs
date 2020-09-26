use std::process::Command;

pub fn get_hs_path() -> String {
  let output = Command::new("wmic")
    .args(&[
      "Process",
      "where",
      "name='Hearthstone.exe'",
      "get",
      "executablepath",
    ])
    .output()
    .expect("failed to execute process");
  let st = String::from_utf8_lossy(&output.stdout);
  println!("{}", st);
  return "测试".to_string();
}
