use std::net::{SocketAddr, TcpStream};
use std::time::Duration;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use serde::{Serialize, Deserialize};
use anyhow::{Result, Context};
use log::info;

// 检查端口是否被占用
pub fn is_port_in_use(host: &str, port: u16) -> bool {
    match format!("{host}:{port}").parse::<SocketAddr>() {
        Ok(addr) => TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_ok(),
        Err(_) => false,
    }
}

// 查找可用端口
pub fn find_available_port(host: &str, start_port: u16) -> Option<u16> {
    for port in start_port..65535 {
        if !is_port_in_use(host, port) {
            return Some(port);
        }
    }
    None
}

// 保存配置到文件
pub fn save_config<T: Serialize>(config: &T, file_path: &str) -> Result<()> {
    let config_dir = Path::new(file_path).parent().unwrap_or(Path::new(""));
    if !config_dir.exists() {
        fs::create_dir_all(config_dir).context("Failed to create config directory")?;
    }
    
    let json = serde_json::to_string_pretty(config).context("Failed to serialize config")?;
    let mut file = File::create(file_path).context("Failed to create config file")?;
    file.write_all(json.as_bytes()).context("Failed to write config file")?;
    
    info!("Configuration saved to {}", file_path);
    Ok(())
}

// 从文件加载配置
pub fn load_config<T: for<'de> Deserialize<'de>>(file_path: &str) -> Result<T> {
    let mut file = File::open(file_path).context("Failed to open config file")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).context("Failed to read config file")?;
    
    let config: T = serde_json::from_str(&contents).context("Failed to parse config file")?;
    info!("Configuration loaded from {}", file_path);
    Ok(config)
}

// 获取应用程序数据目录
pub fn get_app_data_dir() -> Result<String> {
    let home_dir = dirs::home_dir().context("Failed to get home directory")?;
    let app_dir = home_dir.join(".invizible-pro");
    
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir).context("Failed to create app data directory")?;
    }
    
    Ok(app_dir.to_string_lossy().to_string())
}

// 检查应用程序是否以管理员权限运行
pub fn is_running_as_admin() -> bool {
    #[cfg(target_os = "windows")]
    {
        use winapi::um::winnt::{SECURITY_BUILTIN_DOMAIN_RID, DOMAIN_ALIAS_RID_ADMINS};
        use winapi::shared::minwindef::BOOL;
        use winapi::um::winnt::PSID;
        use winapi::um::securitybaseapi::{AllocateAndInitializeSid, CheckTokenMembership, FreeSid};
        use winapi::um::winnt::SID_IDENTIFIER_AUTHORITY;
        use std::ptr::null_mut;
        
        unsafe {
            let mut authority: SID_IDENTIFIER_AUTHORITY = std::mem::zeroed();
            // SECURITY_NT_AUTHORITY 的值为 {0, 0, 0, 0, 0, 5}
            authority.Value[5] = 5;
            
            let mut sid: PSID = null_mut();
            
            // 分配并初始化SID
            let result = AllocateAndInitializeSid(
                &mut authority,
                2,
                SECURITY_BUILTIN_DOMAIN_RID,
                DOMAIN_ALIAS_RID_ADMINS,
                0, 0, 0, 0, 0, 0,
                &mut sid
            );
            
            if result == 0 {
                return false;
            }
            
            // 确保在函数结束时释放SID
            let sid_guard = scopeguard::guard(sid, |sid| {
                FreeSid(sid);
            });
            
            let mut is_member: BOOL = 0;
            let result = CheckTokenMembership(null_mut(), *sid_guard, &mut is_member);
            
            if result == 0 {
                return false;
            }
            
            return is_member != 0;
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

// 格式化字节大小为人类可读的形式
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    format!("{:.2} {}", size, UNITS[unit_index])
}