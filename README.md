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

这是`GPUI`源码中`manifest`清单的: [位置](https://github.com/zed-industries/zed/blob/v0.188.4/crates/gpui/resources/windows/gpui.manifest.xml)

以便程序可以以管理员权限运行。或者你可以fork一份`GPUI`源码使用`Patch`方案, 而不是修改本地源码。