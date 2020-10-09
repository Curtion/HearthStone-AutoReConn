use std::os::windows::process::CommandExt;
use std::process::Command;
fn get_hs_path() -> String {
  const CREATE_NO_WINDOW: u32 = 0x08000000;
  let mut process = Command::new("wmic");
  process
    .args(&[
      "Process",
      "where",
      // "name='Hearthstone.exe'",
      "name='QQ.exe'",
      "get",
      "executablepath",
    ])
    .creation_flags(CREATE_NO_WINDOW);
  let output = process.output().expect("获取路径失败");
  let path = String::from_utf8_lossy(&output.stdout).to_string();
  let path: Vec<&str> = path.split_whitespace().collect();
  path[1].to_string()
}
fn is_fw_rule() -> bool {
  true
}
pub fn start() {
  let hs_path = get_hs_path();
  if is_fw_rule() {
    // 开始任务
  } else {
    // 先创建规则再开始任务
  }
  println!("{}", hs_path);
}
