use eframe::egui::{self, Color32, RichText, Ui, Grid, ScrollArea};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use reqwest::blocking::Client;
use base64::{Engine as _, engine::general_purpose};
use yaml_rust::{YamlLoader, Yaml};
use chrono;

use crate::logger::Logger;

use crate::app::VPN_COLOR;

// VPN协议类型
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VpnProtocol {
    Vmess,
    Shadowsocks,
    Trojan,
    Wireguard,
    OpenVPN,
}

// VPN配置结构
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VpnConfig {
    pub id: usize,
    pub name: String,
    pub protocol: VpnProtocol,
    pub server: String,
    pub port: u16,
    pub uuid: String,
    pub encryption: String,
    pub enabled: bool,
}

impl VpnConfig {
    pub fn new(id: usize, name: &str, protocol: VpnProtocol, server: &str, port: u16, uuid: &str, encryption: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            protocol,
            server: server.to_string(),
            port,
            uuid: uuid.to_string(),
            encryption: encryption.to_string(),
            enabled: false,
        }
    }
}

// Clash订阅结构
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClashSubscription {
    pub id: usize,
    pub name: String,
    pub url: String,
    pub last_updated: String,
    pub configs: Vec<VpnConfig>,
}

impl ClashSubscription {
    pub fn new(id: usize, name: &str, url: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            url: url.to_string(),
            last_updated: "从未".to_string(),
            configs: Vec::new(),
        }
    }
}

// VPN模块结构
pub struct VpnModule {
    enabled: bool,
    configs: Vec<VpnConfig>,
    subscriptions: Vec<ClashSubscription>,
    next_config_id: usize,
    next_subscription_id: usize,
    logger: Arc<Mutex<crate::logger::Logger>>,
    selected_config: Option<usize>,
    selected_subscription: Option<usize>,
    new_config_name: String,
    new_config_protocol: VpnProtocol,
    new_config_server: String,
    new_config_port: u16,
    new_config_uuid: String,
    new_config_encryption: String,
    new_subscription_name: String,
    new_subscription_url: String,
    edit_mode: bool,
    connection_status: String,
    show_subscription_warning: bool,
}

// 修复VpnModule的闭合问题
impl VpnModule {
    pub fn new(logger: Arc<Mutex<Logger>>) -> Self {
        let mut module = Self {
            enabled: false,
            configs: Vec::new(),
            subscriptions: Vec::new(),
            next_config_id: 1,
            next_subscription_id: 1,
            logger,
            selected_config: None,
            selected_subscription: None,
            new_config_name: String::new(),
            new_config_protocol: VpnProtocol::Vmess,
            new_config_server: String::new(),
            new_config_port: 443,
            new_config_uuid: String::new(),
            new_config_encryption: "auto".to_string(),
            new_subscription_name: String::new(),
            new_subscription_url: String::new(),
            edit_mode: false,
            connection_status: "未连接".to_string(),
            show_subscription_warning: false,
        };
        
        // 添加一些示例配置
        module.add_example_configs();
        
        // 记录模块初始化日志
        if let Ok(mut logger) = module.logger.lock() {
            logger.info("VPN", "VPN模块已初始化");
        }
        
        module
    }
    
    // 添加示例配置
    fn add_example_configs(&mut self) {
        // 添加一些示例VPN配置
        let config1 = VpnConfig::new(
            self.next_config_id,
            "示例Vmess服务器",
            VpnProtocol::Vmess,
            "example.com",
            443,
            "a1b2c3d4-e5f6-g7h8-i9j0-k1l2m3n4o5p6",
            "auto"
        );
        self.configs.push(config1);
        self.next_config_id += 1;
        
        let config2 = VpnConfig::new(
            self.next_config_id,
            "示例Shadowsocks服务器",
            VpnProtocol::Shadowsocks,
            "example.org",
            8388,
            "password123",
            "aes-256-gcm"
        );
        self.configs.push(config2);
        self.next_config_id += 1;
    }
    
    // 添加新配置
    fn add_config(&mut self, config: VpnConfig) {
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("VPN", &format!("添加新VPN配置: {}", config.name));
        }
        self.configs.push(config);
        self.next_config_id += 1;
    }
    
    // 删除配置
    fn remove_config(&mut self, id: usize) {
        if let Some(index) = self.configs.iter().position(|c| c.id == id) {
            let config = &self.configs[index];
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("VPN", &format!("删除VPN配置: {}", config.name));
            }
            self.configs.remove(index);
            if self.selected_config == Some(id) {
                self.selected_config = None;
            }
        }
    }
    
    // 添加新订阅
    fn add_subscription(&mut self, subscription: ClashSubscription) {
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("VPN", &format!("添加新Clash订阅: {}", subscription.name));
        }
        self.subscriptions.push(subscription);
        self.next_subscription_id += 1;
    }
    
    // 删除订阅
    fn remove_subscription(&mut self, id: usize) {
        if let Some(index) = self.subscriptions.iter().position(|s| s.id == id) {
            let subscription = &self.subscriptions[index];
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("VPN", &format!("删除Clash订阅: {}", subscription.name));
            }
            self.subscriptions.remove(index);
            if self.selected_subscription == Some(id) {
                self.selected_subscription = None;
            }
        }
    }
    
    // 更新订阅
    fn update_subscription(&mut self, id: usize) {
        if let Some(subscription) = self.subscriptions.iter_mut().find(|s| s.id == id) {
            {
                if let Ok(mut logger) = self.logger.lock() {
                    logger.info("VPN", &format!("正在更新Clash订阅: {}", subscription.name));
                }
            }

            let url = subscription.url.clone();
            match self.download_and_parse_clash_config(&url) {
                Ok(configs) => {
                    let now = chrono::Local::now();
                    subscription.last_updated = now.format("%Y-%m-%d %H:%M:%S").to_string();
                    
                    let mut current_id = self.next_config_id;
                    let new_configs: Vec<VpnConfig> = configs.into_iter()
                        .map(|mut config| {
                            config.id = current_id;
                            current_id += 1;
                            config
                        })
                        .collect();
                    
                    subscription.configs = new_configs;
                    self.next_config_id = current_id;
                    
                    if let Ok(mut logger) = self.logger.lock() {
                        logger.info("VPN", &format!("Clash订阅 {} 已更新，添加了 {} 个配置", 
                                                  subscription.name, subscription.configs.len()));
                    }
                },
                Err(err) => {
                    if let Ok(mut logger) = self.logger.lock() {
                        logger.error("VPN", &format!("更新Clash订阅失败: {}", err));
                    }
                }
            }
        }  // 结束if let块
    }  // 正确闭合update_subscription方法
    
    // 下载并解析Clash配置
    fn download_and_parse_clash_config(&self, url: &str) -> Result<Vec<VpnConfig>, String> {
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("VPN", &format!("正在从 {} 下载Clash配置", url));
        }
        
        // 使用reqwest下载配置
        let client = Client::new();
        let response = match client.get(url).send() {
            Ok(resp) => resp,
            Err(e) => return Err(format!("下载失败: {}", e)),
        };
        
        if !response.status().is_success() {
            return Err(format!("HTTP错误: {}", response.status()));
        }
        
        let content = match response.text() {
            Ok(text) => text,
            Err(e) => return Err(format!("读取响应内容失败: {}", e)),
        };
        
        // 解析YAML
        let docs = match YamlLoader::load_from_str(&content) {
            Ok(docs) => docs,
            Err(e) => return Err(format!("解析YAML失败: {}", e)),
        };
        
        if docs.is_empty() {
            return Err("YAML文档为空".to_string());
        }
        
        let doc = &docs[0];
        
        // 解析代理配置
        let mut configs = Vec::new();
        
        // 尝试获取proxies字段
        if let Some(proxies) = doc["proxies"].as_vec() {
            for (i, proxy) in proxies.iter().enumerate() {
                if let Some(config) = self.parse_clash_proxy(proxy, i) {
                    configs.push(config);
                }
            }
        }
        
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("VPN", &format!("成功解析 {} 个VPN配置", configs.len()));
        }
        
        Ok(configs)
    }
    
    // 解析单个Clash代理配置
    fn parse_clash_proxy(&self, proxy: &Yaml, index: usize) -> Option<VpnConfig> {
        // 处理名称，确保使用String而不是&str
        let name_str = match proxy["name"].as_str() {
            Some(s) => s.to_string(),
            _ => format!("未命名代理{}", index)
        };
        
        // 使用to_string()确保proxy_type是String类型
        let proxy_type = proxy["type"].as_str().unwrap_or("unknown").to_string();
        
        match proxy_type.to_lowercase().as_str() {
            "vmess" => {
                let server = proxy["server"].as_str().unwrap_or("unknown").to_string();
                let port = proxy["port"].as_i64().unwrap_or(443) as u16;
                let uuid = proxy["uuid"].as_str().unwrap_or("").to_string();
                let encryption = proxy["cipher"].as_str().unwrap_or("auto").to_string();
                
                Some(VpnConfig::new(
                    0, // 临时ID，会在调用方重新分配
                    &name_str,
                    VpnProtocol::Vmess,
                    &server,
                    port,
                    &uuid,
                    &encryption
                ))
            },
            "ss" | "shadowsocks" => {
                let server = proxy["server"].as_str().unwrap_or("unknown").to_string();
                let port = proxy["port"].as_i64().unwrap_or(8388) as u16;
                let password = proxy["password"].as_str().unwrap_or("").to_string();
                let encryption = proxy["cipher"].as_str().unwrap_or("aes-256-gcm").to_string();
                
                Some(VpnConfig::new(
                    0, // 临时ID，会在调用方重新分配
                    &name_str,
                    VpnProtocol::Shadowsocks,
                    &server,
                    port,
                    &password,
                    &encryption
                ))
            },
            "trojan" => {
                let server = proxy["server"].as_str().unwrap_or("unknown").to_string();
                let port = proxy["port"].as_i64().unwrap_or(443) as u16;
                let password = proxy["password"].as_str().unwrap_or("").to_string();
                
                Some(VpnConfig::new(
                    0, // 临时ID，会在调用方重新分配
                    &name_str,
                    VpnProtocol::Trojan,
                    &server,
                    port,
                    &password,
                    "auto"
                ))
            },
            _ => None
        }
    }
    
    // 从Base64编码的URL解析Vmess配置
    fn parse_vmess_url(&self, vmess_url: &str) -> Result<VpnConfig, String> {
        // vmess://base64(json)
        if !vmess_url.starts_with("vmess://") {
            return Err("不是有效的Vmess URL".to_string());
        }
        
        let base64_str = &vmess_url[8..]; // 去掉 "vmess://"
        
        // 解码Base64
        let decoded = match general_purpose::STANDARD.decode(base64_str) {
            Ok(bytes) => bytes,
            Err(_) => return Err("Base64解码失败".to_string()),
        };
        
        // 解析JSON
        let json_str = match String::from_utf8(decoded) {
            Ok(s) => s,
            Err(_) => return Err("UTF-8解码失败".to_string()),
        };
        
        // 解析JSON
        let json: serde_json::Value = match serde_json::from_str(&json_str) {
            Ok(v) => v,
            Err(e) => return Err(format!("JSON解析失败: {}", e)),
        };
        
        // 提取配置信息
        let name = json["ps"].as_str().unwrap_or("从URL导入的Vmess");
        let server = json["add"].as_str().unwrap_or("unknown");
        let port_str = json["port"].as_str().unwrap_or("443");
        let port = port_str.parse::<u16>().unwrap_or(443);
        let uuid = json["id"].as_str().unwrap_or("");
        let encryption = json["scy"].as_str().unwrap_or("auto");
        
        let config = VpnConfig::new(
            0, // 临时ID，会在调用方重新分配
            name,
            VpnProtocol::Vmess,
            server,
            port,
            uuid,
            encryption
        );
        
        Ok(config)
    }
    
    // 导入VPN配置URL
    fn import_vpn_url(&mut self, url_str: &str) -> Result<(), String> {
        if url_str.starts_with("vmess://") {
            // 先解析URL，避免同时借用self
            let config_result = self.parse_vmess_url(url_str);
            
            match config_result {
                Ok(config) => {
                    // 获取下一个ID并递增
                    let next_id = self.next_config_id;
                    self.next_config_id += 1;
                    
                    let config_with_id = VpnConfig::new(
                        next_id,
                        &config.name,
                        config.protocol,
                        &config.server,
                        config.port,
                        &config.uuid,
                        &config.encryption
                    );
                    
                    let logger_clone = self.logger.clone();
                    // 记录日志
                    {
                        if let Ok(mut logger) = logger_clone.lock() {
                            logger.info("VPN", &format!("添加新VPN配置: {}", config_with_id.name));
                        }
                    }
                    
                    // 添加配置
                    self.configs.push(config_with_id);
                    Ok(())
                },
                Err(e) => Err(e)
            }
        } else if url_str.starts_with("ss://") {
            // 解析Shadowsocks URL
            // 实现类似parse_vmess_url的功能
            // 解析Shadowsocks URL
let parts: Vec<&str> = url.split('@').collect();
if parts.len() != 2 {
    return Err("无效的Shadowsocks URL".to_string());
}
let method_password = parts[0];
let server_port = parts[1];

let mp: Vec<&str> = method_password.split(':').collect();
if mp.len() != 2 {
    return Err("无效的Shadowsocks加密方法或密码".to_string());
}
let encryption = mp[0];
let password = mp[1];

let sp: Vec<&str> = server_port.split(':').collect();
if sp.len() != 2 {
    return Err("无效的服务器地址或端口".to_string());
}
let server = sp[0];
let port = match sp[1].parse::<u16>() {
    Ok(p) => p,
    Err(_) => return Err("无效的端口号".to_string()),
};

Ok(VpnConfig::new(
    0,
    "Shadowsocks配置",
    VpnProtocol::Shadowsocks,
    server,
    port,
    password,
    encryption
))
        } else if url_str.starts_with("trojan://") {
            // 解析Trojan URL
            // 实现类似parse_vmess_url的功能
            // 解析Trojan URL
let parts: Vec<&str> = url.split('@').collect();
if parts.len() != 2 {
    return Err("无效的Trojan URL".to_string());
}
let password = parts[0];
let server_port = parts[1];

let sp: Vec<&str> = server_port.split(':').collect();
if sp.len() != 2 {
    return Err("无效的服务器地址或端口".to_string());
}
let server = sp[0];
let port = match sp[1].parse::<u16>() {
    Ok(p) => p,
    Err(_) => return Err("无效的端口号".to_string()),
};

Ok(VpnConfig::new(
    0,
    "Trojan配置",
    VpnProtocol::Trojan,
    server,
    port,
    password,
    "auto"
))
        } else {
            Err("不支持的URL格式".to_string())
        }
    }
    
    // 启用/禁用VPN
    fn toggle_vpn(&mut self) {
        // 先获取当前状态的副本，避免同时借用
        let new_enabled = !self.enabled;
        let status_message = if new_enabled { "启用" } else { "禁用" };
        let logger_clone = self.logger.clone();
        
        // 记录日志
        {
            if let Ok(mut logger) = logger_clone.lock() {
                logger.info("VPN", &format!("VPN已{}", status_message));
            }
        }
        
        // 更新状态
        self.enabled = new_enabled;
        self.connection_status = if new_enabled { "正在连接..." } else { "未连接" }.to_string();
        
        // 启动或停止VPN服务
        if new_enabled {
            // 查找启用的配置
            let enabled_configs: Vec<VpnConfig> = self.configs.iter()
                .filter(|c| c.enabled)
                .cloned() // 克隆所有配置避免借用冲突
                .collect();
            
            if enabled_configs.is_empty() {
                {
                    // 使用单独的作用域限制logger的借用范围
                    if let Ok(mut logger) = self.logger.lock() {
                        logger.warning("VPN", "没有启用的VPN配置，无法连接");
                    }
                }
                self.enabled = false;
                self.connection_status = "未连接".to_string();
                return;
            }
            
            // 使用第一个启用的配置
            let config = &enabled_configs[0]; // 已经克隆，不需要再次克隆
            
            // 记录连接信息
            {
                // 使用单独的作用域限制logger的借用范围
                if let Ok(mut logger) = self.logger.lock() {
                    logger.info("VPN", &format!("正在连接到 {} ({}:{})", 
                                                config.name, config.server, config.port));
                }
            }
            
            // 在实际应用中，这里会根据协议类型启动不同的VPN客户端
            match config.protocol {
                VpnProtocol::Vmess => self.start_vmess_client(config),
                VpnProtocol::Shadowsocks => self.start_shadowsocks_client(config),
                VpnProtocol::Trojan => self.start_trojan_client(config),
                VpnProtocol::Wireguard => self.start_wireguard_client(config),
                VpnProtocol::OpenVPN => self.start_openvpn_client(config),
            }
            
            // 模拟连接成功
            self.connection_status = "已连接".to_string();
        } else {
            // 停止VPN客户端
            self.stop_vpn_client();
        }
    }
    
    // 启动Vmess客户端
    fn start_vmess_client(&mut self, config: &VpnConfig) {
        // 克隆必要变量避免借用冲突
        let client_name = config.name.clone();
        let logger_clone = self.logger.clone();
        
        // 在单独作用域中使用克隆的logger
        {
            if let Ok(mut logger) = logger_clone.lock() {
                logger.info("VPN", &format!("启动Vmess客户端: {}", client_name));
            }
        }
        
        // 启动Vmess客户端

        if let Ok(mut logger) = self.logger.lock() {
            logger.info("VPN", &format!("正在启动Vmess客户端: {}", config.name));
        }
        let client = VmessClient::new(config.server.clone(), config.port, config.uuid.clone(), config.encryption.clone());
        match client.connect() {
            Ok(_) => {
                if let Ok(mut logger) = self.logger.lock() {
                    logger.info("VPN", "Vmess客户端启动成功");
                }
            }
            Err(e) => {
                if let Ok(mut logger) = self.logger.lock() {
                    logger.error("VPN", &format!("Vmess客户端启动失败: {}", e));
                }
            }
        }
        // 例如使用v2ray-rust库或调用外部v2ray程序
    }
    
    // 启动Shadowsocks客户端
    fn start_shadowsocks_client(&mut self, config: &VpnConfig) {
        // 克隆必要变量避免借用冲突
        let client_name = config.name.clone();
        let logger_clone = self.logger.clone();
        
        // 在单独作用域中使用克隆的logger
        {
            if let Ok(mut logger) = logger_clone.lock() {
                logger.info("VPN", &format!("启动Shadowsocks客户端: {}", client_name));
            }
        }
        
        // 启动Shadowsocks客户端
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("VPN", &format!("正在启动Shadowsocks客户端: {}", config.name));
        }
        let client = ShadowsocksClient::new(config.server.clone(), config.port, config.uuid.clone(), config.encryption.clone());
        match client.connect() {
            Ok(_) => {
                if let Ok(mut logger) = self.logger.lock() {
                    logger.info("VPN", "Shadowsocks客户端启动成功");
                }
            }
            Err(e) => {
                if let Ok(mut logger) = self.logger.lock() {
                    logger.error("VPN", &format!("Shadowsocks客户端启动失败: {}", e));
                }
            }
        }
    }
    }
    
    // 启动Trojan客户端
    fn start_trojan_client(&mut self, config: &VpnConfig) {
        // 克隆必要变量避免借用冲突
        let client_name = config.name.clone();
        let logger_clone = self.logger.clone();
        
        // 在单独作用域中使用克隆的logger
        {
            if let Ok(mut logger) = logger_clone.lock() {
                logger.info("VPN", &format!("启动Trojan客户端: {}", client_name));
            }
        }
        
        // 在实际应用中，这里会启动Trojan客户端
    }
    
    // 启动Wireguard客户端
    fn start_wireguard_client(&mut self, config: &VpnConfig) {
        // 克隆必要变量避免借用冲突
        let client_name = config.name.clone();
        let logger_clone = self.logger.clone();
        
        // 在单独作用域中使用克隆的logger
        {
            if let Ok(mut logger) = logger_clone.lock() {
                logger.info("VPN", &format!("启动Wireguard客户端: {}", client_name));
            }
        }
        
        // 启动Wireguard客户端
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("VPN", &format!("正在启动Wireguard客户端: {}", config.name));
        }
        let client = WireguardClient::new(config.server.clone(), config.port, config.uuid.clone());
        match client.connect() {
            Ok(_) => {
                if let Ok(mut logger) = self.logger.lock() {
                    logger.info("VPN", "Wireguard客户端启动成功");
                }
            }
            Err(e) => {
                if let Ok(mut logger) = self.logger.lock() {
                    logger.error("VPN", &format!("Wireguard客户端启动失败: {}", e));
                }
            }
        }
    }
    
    // 启动OpenVPN客户端
    fn start_openvpn_client(&mut self, config: &VpnConfig) {
        // 克隆必要变量避免借用冲突
        let client_name = config.name.clone();
        let logger_clone = self.logger.clone();
        
        // 在单独作用域中使用克隆的logger
        {
            if let Ok(mut logger) = logger_clone.lock() {
                logger.info("VPN", &format!("启动OpenVPN客户端: {}", client_name));
            }
        }
        
        // 启动OpenVPN客户端
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("VPN", &format!("正在启动OpenVPN客户端: {}", config.name));
        }
        let client = OpenVPNClient::new(config.server.clone(), config.port, config.uuid.clone());
        match client.connect() {
            Ok(_) => {
                if let Ok(mut logger) = self.logger.lock() {
                    logger.info("VPN", "OpenVPN客户端启动成功");
                }
            }
            Err(e) => {
                if let Ok(mut logger) = self.logger.lock() {
                    logger.error("VPN", &format!("OpenVPN客户端启动失败: {}", e));
                }
            }
        }
    }
    
    // 停止VPN客户端
    fn stop_vpn_client(&mut self) {
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("VPN", "停止VPN客户端");
        }
        
        // 停止所有VPN客户端
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("VPN", "正在停止所有VPN客户端");
        }
        self.configs.iter().for_each(|config| {
            match config.protocol {
                VpnProtocol::Vmess => VmessClient::disconnect(),
                VpnProtocol::Shadowsocks => ShadowsocksClient::disconnect(),
                VpnProtocol::Trojan => TrojanClient::disconnect(),
                VpnProtocol::Wireguard => WireguardClient::disconnect(),
                VpnProtocol::OpenVPN => OpenVPNClient::disconnect(),
            }
        });
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("VPN", "所有VPN客户端已停止");
        }
    }
    
    // 启用/禁用配置
    fn toggle_config(&mut self, id: usize) {
        // 先查找配置并获取必要信息，避免同时借用
        let config_info = self.configs.iter_mut()
            .find(|c| c.id == id)
            .map(|config| {
                let name = config.name.clone();
                let new_state = !config.enabled;
                config.enabled = new_state;
                (name, new_state)
            });
        
        // 如果找到了配置，记录日志
        if let Some((name, enabled)) = config_info {
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("VPN", &format!("VPN配置 '{}' 已{}", name, if enabled { "启用" } else { "禁用" }));
            }
        }
    }
    
    // 显示订阅警告对话框
    fn show_subscription_warning_dialog(&mut self, ui: &mut Ui) -> bool {
        let mut result = false;
        
        egui::Window::new("订阅安全警告")
            .collapsible(false)
            .resizable(false)
            .show(ui.ctx(), |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.heading(RichText::new("安全警告").color(Color32::RED));
                    ui.add_space(10.0);
                });
                
                ui.label("您正在添加一个Clash订阅。请确保您信任此订阅源，因为恶意订阅可能会:");
                ui.add_space(5.0);
                ui.label("1. 将您的流量重定向到恶意服务器");
                ui.label("2. 监控您的网络活动");
                ui.label("3. 收集您的个人信息");
                ui.add_space(10.0);
                ui.label("请仅使用来自可信来源的订阅链接。");
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button("取消").clicked() {
                        self.show_subscription_warning = false;
                    }
                    
                    if ui.button(RichText::new("我了解风险，继续添加").color(Color32::RED)).clicked() {
                        result = true;
                        self.show_subscription_warning = false;
                    }
                });
            });
        
        result
    }
    
    // 渲染UI
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("VPN服务").color(VPN_COLOR).strong());
            ui.add_space(10.0);
            
            let status_text = &self.connection_status;
            let status_color = match status_text.as_str() {
                "已连接" => Color32::GREEN,
                "正在连接..." => Color32::YELLOW,
                _ => Color32::RED,
            };
            ui.label(RichText::new(status_text).color(status_color).strong());
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(if self.enabled { "断开连接" } else { "连接VPN" }).clicked() {
                    self.toggle_vpn();
                }
            });
        });
        
        ui.separator();
        
        // VPN简介
        ui.collapsing("关于VPN服务", |ui| {
            ui.label("VPN（虚拟专用网络）服务允许您通过加密隧道保护您的网络流量。");
            ui.label("这对于访问被地理限制或审查的内容特别有用，同时也能保护您的隐私。");
            ui.label("本模块支持多种VPN协议，包括Vmess、Shadowsocks等，并支持导入Clash订阅。");
        });
        
        ui.separator();
        
        // 如果显示订阅警告对话框
        if self.show_subscription_warning {
            let proceed = self.show_subscription_warning_dialog(ui);
            if proceed {
                // 创建并添加新订阅
                let subscription = ClashSubscription::new(
                    self.next_subscription_id,
                    &self.new_subscription_name,
                    &self.new_subscription_url
                );
                self.add_subscription(subscription);
                
                // 清空输入字段
                self.new_subscription_name.clear();
                self.new_subscription_url.clear();
            }
        }
        
        // 标签页
        egui::TopBottomPanel::top("vpn_tabs").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.edit_mode, false, "VPN配置");
                ui.selectable_value(&mut self.edit_mode, true, "Clash订阅");
            });
        });
        
        if !self.edit_mode {
            // VPN配置列表
            ui.heading("VPN配置列表");
            
            ScrollArea::vertical().show(ui, |ui| {
                for config in &mut self.configs {
                    let is_selected = self.selected_config == Some(config.id);
                    let selected = is_selected;
                    
                    let config_id = config.id;
                    ui.horizontal(|ui| {
                        let mut enabled = config.enabled;
                        if ui.checkbox(&mut enabled, "").clicked() {
                            self.toggle_config(config_id);
                        }
                        
                        if ui.selectable_label(selected, &config.name).clicked() {
                            if is_selected {
                                self.selected_config = None;
                            } else {
                                self.selected_config = Some(config.id);
                            }
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("删除").clicked() {
                                self.remove_config(config.id);
                            }
                        });
                    });
                    
                    if selected {
                        ui.indent("config_details", |ui| {
                            Grid::new(format!("config_grid_{}", config.id))
                                .num_columns(2)
                                .spacing([10.0, 8.0])
                                .show(ui, |ui| {
                                    ui.label("协议:");
                                    ui.label(format!("{:?}", config.protocol));
                                    ui.end_row();
                                    
                                    ui.label("服务器:");
                                    ui.label(&config.server);
                                    ui.end_row();
                                    
                                    ui.label("端口:");
                                    ui.label(config.port.to_string());
                                    ui.end_row();
                                    
                                    ui.label("UUID/密码:");
                                    ui.label(&config.uuid);
                                    ui.end_row();
                                    
                                    ui.label("加密方式:");
                                    ui.label(&config.encryption);
                                    ui.end_row();
                                });
                        });
                    }
                }
            });
            
            ui.separator();
            
            // 添加新VPN配置
            ui.heading("添加新VPN配置");
            
            Grid::new("new_config_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label("名称:");
                    ui.text_edit_singleline(&mut self.new_config_name);
                    ui.end_row();
                    
                    ui.label("协议:");
                    egui::ComboBox::from_id_source("protocol_combo")
                        .selected_text(format!("{:?}", self.new_config_protocol))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.new_config_protocol, VpnProtocol::Vmess, "Vmess");
                            ui.selectable_value(&mut self.new_config_protocol, VpnProtocol::Shadowsocks, "Shadowsocks");
                            ui.selectable_value(&mut self.new_config_protocol, VpnProtocol::Trojan, "Trojan");
                            ui.selectable_value(&mut self.new_config_protocol, VpnProtocol::Wireguard, "Wireguard");
                            ui.selectable_value(&mut self.new_config_protocol, VpnProtocol::OpenVPN, "OpenVPN");
                        });
                    ui.end_row();
                    
                    ui.label("服务器:");
                    ui.text_edit_singleline(&mut self.new_config_server);
                    ui.end_row();
                    
                    ui.label("端口:");
                    let mut port_str = self.new_config_port.to_string();
                    if ui.text_edit_singleline(&mut port_str).changed() {
                        if let Ok(port) = port_str.parse::<u16>() {
                            self.new_config_port = port;
                        }
                    }
                    ui.end_row();
                    
                    ui.label("UUID/密码:");
                    ui.text_edit_singleline(&mut self.new_config_uuid);
                    ui.end_row();
                    
                    ui.label("加密方式:");
                    ui.text_edit_singleline(&mut self.new_config_encryption);
                    ui.end_row();
                });
            
            if ui.button("添加配置").clicked() {
                if !self.new_config_name.is_empty() && !self.new_config_server.is_empty() && !self.new_config_uuid.is_empty() {
                    let config = VpnConfig::new(
                        self.next_config_id,
                        &self.new_config_name,
                        self.new_config_protocol.clone(),
                        &self.new_config_server,
                        self.new_config_port,
                        &self.new_config_uuid,
                        &self.new_config_encryption
                    );
                    self.add_config(config);
                    
                    // 清空输入字段
                    self.new_config_name.clear();
                    self.new_config_server.clear();
                    self.new_config_uuid.clear();
                    self.new_config_encryption = "auto".to_string();
                }
            }
            
            // 导入VPN URL
            ui.separator();
            ui.heading("导入VPN URL");
            
            ui.horizontal(|ui| {
                ui.label("URL:");
                let mut import_url = String::new();
ui.text_edit_singleline(&mut import_url);
                
                if ui.button("导入").clicked() && !import_url.is_empty() {
                    match self.import_vpn_url(&import_url) {
                        Ok(_) => {
                            if let Ok(mut logger) = self.logger.lock() {
                                logger.info("VPN", "成功导入VPN配置");
                            }
                        },
                        Err(e) => {
                            if let Ok(mut logger) = self.logger.lock() {
                                logger.error("VPN", &format!("导入VPN配置失败: {}", e));
                            }
                        }
                    }
                }
            });
            
            ui.label("支持的URL格式: vmess://, ss://, trojan://");
            ui.label("或者直接粘贴分享链接");
        } else {
            // Clash订阅管理
            ui.heading("Clash订阅管理");
            
            ScrollArea::vertical().show(ui, |ui| {
                for subscription in &mut self.subscriptions {
                    let is_selected = self.selected_subscription == Some(subscription.id);
                    let selected = is_selected;
                    
                    let subscription = subscription.clone(); // 克隆订阅以避免多重可变借用
                    let sub_id = subscription.id;
                    ui.horizontal(|ui| {
                        if ui.selectable_label(selected, &subscription.name).clicked() {
                            if is_selected {
                                self.selected_subscription = None;
                            } else {
                                self.selected_subscription = Some(sub_id);
                            }
                        }
                        
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("删除").clicked() {
                                self.remove_subscription(subscription.id);
                            }
                            
                            if ui.button("更新").clicked() {
                                self.update_subscription(subscription.id);
                            }
                        });
                    });
                    
                    if selected {
                        ui.indent("subscription_details", |ui| {
                            Grid::new(format!("subscription_grid_{}", subscription.id))
                                .num_columns(2)
                                .spacing([10.0, 8.0])
                                .show(ui, |ui| {
                                    ui.label("URL:");
                                    ui.label(&subscription.url);
                                    ui.end_row();
                                    
                                    ui.label("最后更新:");
                                    ui.label(&subscription.last_updated);
                                    ui.end_row();
                                    
                                    ui.label("配置数量:");
                                    ui.label(subscription.configs.len().to_string());
                                    ui.end_row();
                                });
                            
                            if !subscription.configs.is_empty() {
                                ui.label("包含的配置:");
                                for config in &subscription.configs {
                                    ui.label(format!("- {} ({:?})", config.name, config.protocol));
                                }
                            }
                        });
                    }
                }
            });
            
            ui.separator();
            
            // 添加新Clash订阅
            ui.heading("添加新Clash订阅");
            
            Grid::new("new_subscription_grid")
                .num_columns(2)
                .spacing([10.0, 8.0])
                .show(ui, |ui| {
                    ui.label("名称:");
                    ui.text_edit_singleline(&mut self.new_subscription_name);
                    ui.end_row();
                    
                    ui.label("URL:");
                    ui.text_edit_singleline(&mut self.new_subscription_url);
                    ui.end_row();
                });
            
            if ui.button("添加订阅").clicked() {
                if !self.new_subscription_name.is_empty() && !self.new_subscription_url.is_empty() {
                    // 显示安全警告对话框
                    self.show_subscription_warning = true;
                }
            }
        }
    }