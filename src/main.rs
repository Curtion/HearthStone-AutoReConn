extern crate clap;
mod program;

use clap::{App, Arg};

fn main() {
    let matches = App::new("hs")
        .version("0.1.0")
        .author("Curtion. curtion@126.com")
        .about("炉石传说酒馆战棋拔线工具！")
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .value_name("NAME")
                .about("可选，防火墙规则名称")
                .takes_value(true),
        )
        .arg(
            Arg::new("program")
                .short('p')
                .long("program")
                .value_name("PATH")
                .about("可选，炉石执行程序路径")
                .takes_value(true),
        )
        .arg(
            Arg::new("second")
                .short('s')
                .long("second")
                .value_name("TIME")
                .about("可选，网络恢复时间(默认3S)")
                .takes_value(true),
        )
        .get_matches();
    let hs_path = program::get_hs_path();
    let name = matches.value_of("name").unwrap_or("Curtion_LS");
    let program = matches.value_of("program").unwrap_or(&hs_path).trim();
    let second = matches.value_of("second").unwrap_or("3");
    println!("{}", name);
    println!("{}", program);
    println!("{}", second);
}
