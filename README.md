# 构建步骤说明

1. Git 克隆本项目
2. 构建：`cargo build --release`
3. `target/release/hsarec.exe` 即为可执行文件文件

# 使用说明

1. [下载压缩包](https://github.com/Curtion/HearthStone-AutoReConn/releases)、解压
2. 双击运行 hsarec.exe
3. 右下角托盘菜单中可选择拔线操作，或者使用快捷键`Shift+Alt+R`快速拔线，目前重连速度为 3s

如果在断线过程中程序意外退出，可能会导致炉石无法连接网络，此时只需要重新运行 hsarec.exe 一次即可解决。

# 申明

本程序不修改炉石传说游戏任何数据, 当前拔线使用`iphlpapi.dll`实现

# 关于`manifest`清单

如果你需要自行编译才需要关注以下内容:

由于使用了`GPUI`框架，框架内会嵌入`manifest`清单文件， 当前程序的`mainfest.rc`中的`2 RT_MANIFEST "auc.manifest"`实际是无效的, 因为它的ID为2。

一旦修改为1会导致和GPUI中的清单冲突，导致编译失败, 因此临时的解决办法如下:

临时的解决方案是:

找到本地源码, 在原清单中加入
```xml
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">  
    <security>  
      <requestedPrivileges>  
        <requestedExecutionLevel level='requireAdministrator' uiAccess='false' />  
      </requestedPrivileges>  
    </security>  
  </trustInfo>
```

以便程序可以以管理员权限运行。或者你可以fork一份`GPUI`源码加入依赖, 而不是修改本地源码。