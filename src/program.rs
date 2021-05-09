use std::os::windows::process::CommandExt;
use std::process::Command;
use std::{thread, time};
use textcode::gb2312;
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn get_hs_path() -> String {
  // 获取炉石执行程序路径
  info!("开始获取炉石路径");
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
  println!("获取到炉石路径:{}", path);
  let path: Vec<&str> = path.split_whitespace().collect();
  info!("炉石路径获取成功:{:?}", path);
  path[1].to_string()
}

pub fn is_fw_rule() -> bool {
  // 判断规则是否存在
  info!("开始判断防火墙规则是否存在");
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
    println!("判断路径:{}", true);
    info!("判断防火墙规则是否存在结束:{}", true);
    true
  } else {
    println!("判断路径:{}", false);
    info!("判断防火墙规则是否存在结束:{}", false);
    false
  }
}

fn creat_firewall_rule(hs_path: &str) {
  // 创建规则
  info!("创建防火墙规则开始");
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
    .arg("enable=NO")
    .creation_flags(CREATE_NO_WINDOW);
  let output = process.output().expect("创建规则失败");
  let mut res = String::new();
  gb2312::decode(&output.stdout, &mut res);
  println!("创建规则:{}", res);
  info!("创建防火墙规则结束:{}", res);
}

pub fn disable() {
  // 恢复网络
  info!("恢复网络开始");
  let mut process = Command::new("netsh");
  process
    .arg("advfirewall")
    .arg("firewall")
    .arg("set")
    .arg("rule")
    .arg("name=Curtion_LS")
    .arg("new")
    .arg("enable=NO")
    .creation_flags(CREATE_NO_WINDOW);
  let output = process.output().expect("恢复网络失败");
  let mut res = String::new();
  gb2312::decode(&output.stdout, &mut res);
  println!("恢复网络:{}", res);
  info!("恢复网络结束:{}", res.trim());
}

fn enable() {
  // 禁用网络
  info!("禁用网络开始");
  let mut process = Command::new("netsh");
  process
    .arg("advfirewall")
    .arg("firewall")
    .arg("set")
    .arg("rule")
    .arg("name=Curtion_LS")
    .arg("new")
    .arg("enable=YES")
    .creation_flags(CREATE_NO_WINDOW);
  let output = process.output().expect("禁用网络失败");
  let mut res = String::new();
  gb2312::decode(&output.stdout, &mut res);
  println!("禁用网络:{}", res);
  info!("禁用网络结束:{}", res);
}

fn start_reconnection() {
  info!("炉石重连开始");
  enable();
  let time = time::Duration::from_millis(3000);
  thread::sleep(time);
  disable();
  info!("炉石重连结束");
}

pub fn start() {
  if !is_fw_rule() {
    // 先创建规则再开始任务
    let hs_path = get_hs_path();
    creat_firewall_rule(&hs_path);
  }
  start_reconnection();
}
