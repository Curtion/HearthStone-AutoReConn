# 构建步骤说明

1. Git 克隆本项目
2. 构建：`cargo build --release`
3. `target/release/hsarec.exe` 即为可执行文件文件

# 使用说明

1. [下载压缩包](https://github.com/Curtion/HearthStone-AutoReConn/releases)、解压
2. 双击运行 hsarec.exe
3. 右下角托盘菜单中可选择拔线操作，或者使用快捷键`Shift+Alt+R`快速拔线
4. 可观察托盘图标拔线过程是否有变化, 如果没有任何变化且拔线无效可以附带日志`hsarec.log`进行反馈。

建议一局中最多拔线10次，超过有概率无法重连回去(据说)

# 申明

本程序不修改炉石传说游戏任何数据, 当前拔线使用`iphlpapi.dll`实现
