use anyhow::Result;
use windows::Win32::NetworkManagement::IpHelper::{
    GetExtendedTcpTable, MIB_TCPTABLE_OWNER_MODULE, TCP_TABLE_OWNER_MODULE_ALL,
};
use windows::Win32::Networking::WinSock::{IN_ADDR, ntohs};

#[derive(Debug)]
pub struct NetworkInfo {
    pub pid: u32,
    pub local_addr: String,
    pub local_port: u16,
    pub remote_addr: String,
    pub remote_port: u16,
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
            let entry = unsafe { &*table_ptr.add(i as usize) };
            
            // 只处理匹配的PID
            if entry.dwOwningPid == pid {
                let local_addr = unsafe {
                    IN_ADDR {
                        S_un: std::mem::transmute(entry.dwLocalAddr),
                    }
                };
                let remote_addr = unsafe {
                    IN_ADDR {
                        S_un: std::mem::transmute(entry.dwRemoteAddr),
                    }
                };

                // 将网络字节序转换为主机字节序
                let local_port = unsafe { ntohs(entry.dwLocalPort as u16) };
                let remote_port = unsafe { ntohs(entry.dwRemotePort as u16) };

                let local_addr_str = unsafe {
                    format!(
                        "{}.{}.{}.{}",
                        local_addr.S_un.S_un_b.s_b1,
                        local_addr.S_un.S_un_b.s_b2,
                        local_addr.S_un.S_un_b.s_b3,
                        local_addr.S_un.S_un_b.s_b4
                    )
                };

                let remote_addr_str = unsafe {
                    format!(
                        "{}.{}.{}.{}",
                        remote_addr.S_un.S_un_b.s_b1,
                        remote_addr.S_un.S_un_b.s_b2,
                        remote_addr.S_un.S_un_b.s_b3,
                        remote_addr.S_un.S_un_b.s_b4
                    )
                };

                network_infos.push(NetworkInfo {
                    pid: entry.dwOwningPid,
                    local_addr: local_addr_str,
                    local_port,
                    remote_addr: remote_addr_str,
                    remote_port,
                });
            }
        }
    }

    Ok(network_infos)
}
