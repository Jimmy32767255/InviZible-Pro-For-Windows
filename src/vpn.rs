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
            let parse_result = self.parse_shadowsocks_url(url_str);
            match parse_result {
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
        } else if url_str.starts_with("trojan://") {
            // 解析Trojan URL
            // 实现类似parse_vmess_url的功能
            let parse_result = self.parse_trojan_url(url_str);
            match parse_result {
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
            ui.heading(RichText::new("VPN").color(VPN_COLOR).strong());
            ui.add_space(10.0);
            
            let status_text = &self.connection_status;
            let status_color = match status_text.as_str() {
                "已连接" => Color32::GREEN,
                "正在连接..." => Color32::YELLOW,
                _ => Color32::RED,
            };
            ui.label(RichText::new(status_text).color(status_color).strong());
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(if self.enabled { "断开VPN" } else { "连接VPN" }).clicked() {
                    self.toggle_vpn();
                }
            });
        });
        
        ui.separator();
        
        // VPN简介
        ui.collapsing("关于VPN", |ui| {
            ui.label("VPN（虚拟私人网络）可以加密您的网络连接，保护您的隐私，并帮助您绕过网络限制。");
            ui.label("本模块支持多种VPN协议，包括Vmess、Shadowsocks、Trojan等。");
            ui.label("您可以手动添加配置，或者通过Clash订阅批量导入配置。");
        });
        
        ui.separator();
        
        // 标签页
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.selected_subscription, None, "VPN配置");
            
            // 显示订阅标签
            for subscription in &self.subscriptions {
                ui.selectable_value(&mut self.selected_subscription, Some(subscription.id), &subscription.name);
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("添加订阅").clicked() {
                    self.edit_mode = true;
                    self.selected_subscription = None;
                }
            });
        });
        
        ui.separator();
        
        // 根据选择的标签页显示内容
        if let Some(subscription_id) = self.selected_subscription {
            // 显示订阅内容
            if let Some(subscription) = self.subscriptions.iter().find(|s| s.id == subscription_id) {
                ui.horizontal(|ui| {
                    ui.heading(&subscription.name);
                    ui.label(format!("(上次更新: {})", subscription.last_updated));
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("更新").clicked() {
                            self.update_subscription(subscription_id);
                        }
                        if ui.button("删除").clicked() {
                            self.remove_subscription(subscription_id);
                        }
                    });
                });
                
                ui.label(format!("URL: {}", subscription.url));
                ui.label(format!("配置数量: {}", subscription.configs.len()));
                
                // 显示订阅中的配置列表
                self.render_config_list(ui, &subscription.configs);
            }
        } else {
            // 显示手动添加的配置
            ui.horizontal(|ui| {
                ui.heading("VPN配置");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("添加配置").clicked() {
                        self.edit_mode = true;
                    }
                });
            });
            
            // 显示配置列表
            self.render_config_list(ui, &self.configs);
        }
    }
    
    // 渲染配置列表
    fn render_config_list(&mut self, ui: &mut Ui, configs: &[VpnConfig]) {
        ScrollArea::vertical().show(ui, |ui| {
            Grid::new("vpn_configs_grid")
                .num_columns(4)
                .striped(true)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    // 表头
                    ui.label(RichText::new("启用").strong());
                    ui.label(RichText::new("名称").strong());
                    ui.label(RichText::new("服务器").strong());
                    ui.label(RichText::new("操作").strong());
                    ui.end_row();
                    
                    // 配置列表
                    let configs_clone = configs.to_vec(); // 克隆配置列表以避免借用冲突
                    for config in &configs_clone {
                        // 启用/禁用复选框
                        let mut enabled = config.enabled;
                        let config_id = config.id; // 先获取ID避免借用冲突
                        if ui.checkbox(&mut enabled, "").changed() {
                            self.toggle_config(config_id);
                        }
                        
                        // 配置名称
                        let config_text = RichText::new(&config.name);
                        if ui.selectable_label(self.selected_config == Some(config.id), config_text).clicked() {
                            self.selected_config = Some(config.id);
                        }
                        
                        // 服务器信息
                        ui.label(format!("{} ({})", config.server, config.port));
                        
                        // 操作按钮
                        let config_id = config.id; // 再次获取ID避免闭包中的借用冲突
                        ui.horizontal(|ui| {
                            if ui.button("编辑").clicked() {
                                // 编辑配置逻辑
                                self.selected_config = Some(config_id);
                                self.edit_mode = true;
                            }
                            if ui.button("删除").clicked() {
                                self.remove_config(config_id);
                            }
                        });
                        
                        ui.end_row();
                    }
                });
        });
    }

    // 添加/编辑配置对话框
    if self.edit_mode {
        let title = if self.selected_subscription.is_some() {
            "添加Clash订阅"
        } else if self.selected_config.is_some() {
            "编辑VPN配置"
        } else {
            "添加VPN配置"
        };
        
        let response = egui::Window::new(title)
            .open(&mut self.edit_mode)
            .show(ui.ctx(), |ui| {
                if self.selected_subscription.is_some() {
                    // 添加Clash订阅表单
                    ui.horizontal(|ui| {
                        ui.label("订阅名称:");
                        ui.text_edit_singleline(&mut self.new_subscription_name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("订阅URL:");
                        ui.text_edit_singleline(&mut self.new_subscription_url);
                    });
                    
                    if self.show_subscription_warning {
                        ui.label(RichText::new("警告: 从不受信任的来源添加订阅可能存在安全风险。").color(Color32::RED));
                    }
                    
                    ui.checkbox(&mut self.show_subscription_warning, "我了解添加订阅的风险");
                    
                    ui.horizontal(|ui| {
                        if ui.button("取消").clicked() {
                            false
                        } else if ui.button("添加").clicked() && self.show_subscription_warning {
                            true
                        } else {
                            false
                        }
                    }).inner.unwrap_or(false)
                } else {
                    // 添加/编辑VPN配置表单
                    ui.horizontal(|ui| {
                        ui.label("配置名称:");
                        ui.text_edit_singleline(&mut self.new_config_name);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("协议类型:");
                        egui::ComboBox::from_id_source("protocol_combo")
                            .selected_text(match self.new_config_protocol {
                                VpnProtocol::Vmess => "Vmess",
                                VpnProtocol::Shadowsocks => "Shadowsocks",
                                VpnProtocol::Trojan => "Trojan",
                                VpnProtocol::Wireguard => "Wireguard",
                                VpnProtocol::OpenVPN => "OpenVPN",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.new_config_protocol, VpnProtocol::Vmess, "Vmess");
                                ui.selectable_value(&mut self.new_config_protocol, VpnProtocol::Shadowsocks, "Shadowsocks");
                                ui.selectable_value(&mut self.new_config_protocol, VpnProtocol::Trojan, "Trojan");
                                ui.selectable_value(&mut self.new_config_protocol, VpnProtocol::Wireguard, "Wireguard");
                                ui.selectable_value(&mut self.new_config_protocol, VpnProtocol::OpenVPN, "OpenVPN");
                            });
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("服务器地址:");
                        ui.text_edit_singleline(&mut self.new_config_server);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("端口:");
                        ui.add(egui::DragValue::new(&mut self.new_config_port).speed(1.0));
                    });
                    
                    ui.horizontal(|ui| {
                        let field_name = match self.new_config_protocol {
                            VpnProtocol::Vmess => "UUID:",
                            VpnProtocol::Shadowsocks | VpnProtocol::Trojan => "密码:",
                            _ => "密钥:",
                        };
                        ui.label(field_name);
                        ui.text_edit_singleline(&mut self.new_config_uuid);
                    });
                    
                    if self.new_config_protocol == VpnProtocol::Vmess || self.new_config_protocol == VpnProtocol::Shadowsocks {
                        ui.horizontal(|ui| {
                            ui.label("加密方式:");
                            ui.text_edit_singleline(&mut self.new_config_encryption);
                        });
                    }
                    
                    ui.horizontal(|ui| {
                        if ui.button("取消").clicked() {
                            false
                        } else if ui.button("保存").clicked() {
                            true
                        } else {
                            false
                        }
                    }).inner.unwrap_or(false)
                }
            });
        
        if let Some(response) = response {
            if response.inner {
                if self.selected_subscription.is_some() {
                    // 添加新订阅
                    if !self.new_subscription_name.is_empty() && !self.new_subscription_url.is_empty() {
                        let new_subscription = ClashSubscription::new(
                            self.next_subscription_id,
                            &self.new_subscription_name,
                            &self.new_subscription_url
                        );
                        self.add_subscription(new_subscription);
                        self.new_subscription_name.clear();
                        self.new_subscription_url.clear();
                    }
                } else {
                    // 添加/编辑VPN配置
                    if !self.new_config_name.is_empty() && !self.new_config_server.is_empty() && !self.new_config_uuid.is_empty() {
                        let new_config = VpnConfig::new(
                            self.next_config_id,
                            &self.new_config_name,
                            self.new_config_protocol.clone(),
                            &self.new_config_server,
                            self.new_config_port,
                            &self.new_config_uuid,
                            &self.new_config_encryption
                        );
                        self.add_config(new_config);
                        self.new_config_name.clear();
                        self.new_config_server.clear();
                        self.new_config_uuid.clear();
                        self.new_config_encryption = "auto".to_string();
                    }
                }
            }
        }
    }
    
    // 配置详情区域
    if let Some(config_id) = self.selected_config {
        if let Some(config) = self.configs.iter().find(|c| c.id == config_id) {
            ui.separator();
            ui.heading("配置详情");
            
            Grid::new("config_details_grid")
                .num_columns(2)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    ui.label("名称:");
                    ui.label(&config.name);
                    ui.end_row();
                    
                    ui.label("协议:");
                    ui.label(match config.protocol {
                        VpnProtocol::Vmess => "Vmess",
                        VpnProtocol::Shadowsocks => "Shadowsocks",
                        VpnProtocol::Trojan => "Trojan",
                        VpnProtocol::Wireguard => "Wireguard",
                        VpnProtocol::OpenVPN => "OpenVPN",
                    });
                    ui.end_row();
                    
                    ui.label("服务器:");
                    ui.label(&config.server);
                    ui.end_row();
                    
                    ui.label("端口:");
                    ui.label(config.port.to_string());
                    ui.end_row();
                    
                    ui.label(match config.protocol {
                        VpnProtocol::Vmess => "UUID:",
                        VpnProtocol::Shadowsocks | VpnProtocol::Trojan => "密码:",
                        _ => "密钥:",
                    });
                    ui.label(&config.uuid);
                    ui.end_row();
                    
                    if config.protocol == VpnProtocol::Vmess || config.protocol == VpnProtocol::Shadowsocks {
                        ui.label("加密方式:");
                        ui.label(&config.encryption);
                        ui.end_row();
                    }
                });
        }
    }