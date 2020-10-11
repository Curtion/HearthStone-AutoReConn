# 构建步骤说明

1. Git克隆本项目
2. 执行`cargo build --release`命令
3. 复制`./target/release/hsarec.exe`文件至`./doc`目录
4. 使用Windows SDK中的`mt.exe` 执行命令：`mt -manifest "uac.manifest" -outputresource:"hsarec.exe"`
5. 复制`hsarec.exe`和`hsarec.ico`到任意处即可双击运行

mt.exe 参考：[https://docs.microsoft.com/en-us/windows/win32/sbscs/mt-exe](https://docs.microsoft.com/en-us/windows/win32/sbscs/mt-exe)

第4步是为了给执行文件添加管理员权限，如果本机没有mt.exe也可以省略此步骤，作为代价就是hsarec.exe需要收集右键选择：以管理员身份运行。

# 使用说明

1. [下载压缩包](https://github.com/Curtion/HearthStone-AutoReConn/releases)、解压
2. 双击运行hsarec.exe
3. 右下角托盘菜单中可选择拔线操作，或者使用快捷键`Shift+Alt+R`快速拔线，目前重连速度为3s

如果在断线过程中程序意外退出，可能会导致炉石无法连接网络，此时只需要重新运行hsarec.exe一次即可解决。

# 说明

作为一款rust练手项目，写得一言难尽，所以本项目只处于简单测试可用状态。

# 申明

本程序不修改炉石传说游戏任何数据.