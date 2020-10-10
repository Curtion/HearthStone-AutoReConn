use std::os::windows::process::CommandExt;
use std::process::Command;
use std::{thread, time};
use textcode::gb2312;
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn get_hs_path() -> String {
  // 获取炉石执行程序路径
  let mut process = Command::new("wmic");
  process
    .arg("Process")
    .arg("where")
    .arg("name='Hearthstone.exe'")
    .arg("get")
    .arg("executablepath")
    .creation_flags(CREATE_NO_WINDOW);
  let output = process.output().expect("获取路径失败");
  let mut path = String::new();
  gb2312::decode(&output.stdout, &mut path);
  let path: Vec<&str> = path.split_whitespace().collect();
  path[1].to_string()
}

fn is_fw_rule() -> bool {
  // 判断规则是否存在
  let mut process = Command::new("netsh");
  process
    .arg("advfirewall")
    .arg("firewall")
    .arg("show")
    .arg("rule")
    .arg("name=Curtion_LS")
    .creation_flags(CREATE_NO_WINDOW);
  let output = process.output().expect("判断路径失败");
  let mut res = String::new();
  gb2312::decode(&output.stdout, &mut res);
  if res.contains("Curtion_LS") {
    true
  } else {
    false
  }
}

fn creat_firewall_rule(hs_path: &str) {
  // 创建规则
  let mut process = Command::new("netsh");
  process
    .arg("advfirewall")
    .arg("firewall")
    .arg("add")
    .arg("rule")
    .arg("name=Curtion_LS")
    .arg("dir=out")
    .arg("action=block")
    .arg(String::new() + "program=" + hs_path)
    .arg("enable=NO");
  process.output().expect("创建规则失败");
}

fn start_reconnection() {
  let mut process = Command::new("netsh");
  process
    .arg("advfirewall")
    .arg("firewall")
    .arg("set")
    .arg("rule")
    .arg("name=Curtion_LS")
    .arg("new")
    .arg("enable=YES");
  process.output().expect("禁用网络失败");
  let time = time::Duration::from_millis(3000);
  thread::sleep(time);
  let mut process = Command::new("netsh");
  process
    .arg("advfirewall")
    .arg("firewall")
    .arg("set")
    .arg("rule")
    .arg("name=Curtion_LS")
    .arg("new")
    .arg("enable=NO");
  process.output().expect("恢复网络失败");
}

pub fn start() {
  if !is_fw_rule() {
    // 先创建规则再开始任务
    let hs_path = get_hs_path();
    creat_firewall_rule(&hs_path);
  }
  start_reconnection();
}
