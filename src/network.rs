use anyhow::Result;
use std::fmt;
use std::net::Ipv4Addr;
use windows::Win32::Foundation::{NO_ERROR, WIN32_ERROR};
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCP_STATE_DELETE_TCB, MIB_TCPROW_LH, MIB_TCPROW_LH_0,
    MIB_TCPTABLE_OWNER_MODULE, SetTcpEntry, TCP_TABLE_OWNER_MODULE_ALL,
};

pub struct NetworkInfo {
    pub local_addr: u32,
    pub local_port: u32,
    pub remote_addr: u32,
    pub remote_port: u32,
}

impl NetworkInfo {
    /// 将 local_addr (u32) 转换为 Ipv4Addr
    /// Windows API 返回的 IP 地址是网络字节序的 u32
    pub fn local_addr_as_ipv4(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.local_addr.to_be())
    }

    /// 将 local_port (u32) 转换为 u16
    /// Windows API 返回的端口是网络字节序的 u32，实际值在低16位
    pub fn local_port_as_u16(&self) -> u16 {
        (self.local_port.to_be() >> 16) as u16
    }

    /// 将 remote_addr (u32) 转换为 Ipv4Addr
    /// Windows API 返回的 IP 地址是网络字节序的 u32
    pub fn remote_addr_as_ipv4(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.remote_addr.to_be())
    }

    /// 将 remote_port (u32) 转换为 u16
    /// Windows API 返回的端口是网络字节序的 u32，实际值在低16位
    pub fn remote_port_as_u16(&self) -> u16 {
        (self.remote_port.to_be() >> 16) as u16
    }
}

impl fmt::Display for NetworkInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{} -> {}:{}",
            self.local_addr_as_ipv4(),
            self.local_port_as_u16(),
            self.remote_addr_as_ipv4(),
            self.remote_port_as_u16()
        )
    }
}

pub fn get_process_by_pid(pid: u32) -> Result<Vec<NetworkInfo>> {
    let mut size: u32 = 0;
    let result: u32 = unsafe {
        GetExtendedTcpTable(
            None,
            &mut size as *mut u32,
            true,
            2,
            TCP_TABLE_OWNER_MODULE_ALL,
            0,
        )
    };
    anyhow::ensure!(
        result == 122 || result == 0,
        "GetExtendedTcpTable异常[1]: {:?}",
        result
    );

    let mut buffer = vec![0u8; size as usize];

    let result = unsafe {
        GetExtendedTcpTable(
            Some(buffer.as_mut_ptr() as *mut _),
            &mut size,
            true,
            2,
            TCP_TABLE_OWNER_MODULE_ALL,
            0,
        )
    };

    anyhow::ensure!(result == 0, "GetExtendedTcpTable异常[2]: {:?}", result);

    buffer.truncate(size as usize);

    let mut network_infos = Vec::new();

    // 解析数据
    if size > 0 {
        let tcp_table = unsafe { &*(buffer.as_ptr() as *const MIB_TCPTABLE_OWNER_MODULE) };

        let table_ptr = tcp_table.table.as_ptr();

        for i in 0..tcp_table.dwNumEntries {
            let entry = unsafe { &*table_ptr.add(i as usize) }; // 只处理匹配的PID
            if entry.dwOwningPid == pid {
                network_infos.push(NetworkInfo {
                    local_addr: entry.dwLocalAddr,
                    local_port: entry.dwLocalPort,
                    remote_addr: entry.dwRemoteAddr,
                    remote_port: entry.dwRemotePort,
                });
            }
        }
    }

    Ok(network_infos)
}

pub fn close_tcp_connection(network_info: &NetworkInfo) -> Result<()> {
    unsafe {
        let tcp_row = MIB_TCPROW_LH {
            Anonymous: MIB_TCPROW_LH_0 {
                State: MIB_TCP_STATE_DELETE_TCB,
            },
            dwLocalAddr: network_info.local_addr,
            dwLocalPort: network_info.local_port,
            dwRemoteAddr: network_info.remote_addr,
            dwRemotePort: network_info.remote_port,
        };

        let result = SetTcpEntry(&tcp_row);
        match WIN32_ERROR(result) {
            NO_ERROR => Ok(()),
            _ => Err(anyhow::anyhow!("关闭TCP连接失败, 错误代码: {}", result)),
        }
    }
}
