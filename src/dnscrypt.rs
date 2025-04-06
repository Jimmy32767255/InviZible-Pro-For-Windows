use eframe::egui::{self, Color32, RichText, Ui, Grid, ScrollArea};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

use crate::logger::Logger;
use crate::app::DNS_COLOR;

// DNSCrypt服务器结构
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DnsCryptServer {
    pub id: usize,
    pub name: String,
    pub address: String,
    pub provider_name: String,
    pub description: String,
    pub enabled: bool,
    pub dnssec: bool,
    pub no_logs: bool,
}

impl DnsCryptServer {
    pub fn new(id: usize, name: &str, address: &str, provider_name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            address: address.to_string(),
            provider_name: provider_name.to_string(),
            description: String::new(),
            enabled: true,
            dnssec: false,
            no_logs: false,
        }
    }
}

// DNSCrypt模块结构
pub struct DnsCryptModule {
    enabled: bool,
    servers: Vec<DnsCryptServer>,
    next_server_id: usize,
    logger: Arc<Mutex<Logger>>,
    selected_server: Option<usize>,
    new_server_name: String,
    new_server_address: String,
    new_server_provider: String,
    edit_mode: bool,
    connection_status: String,
    dns_leak_protection: bool,
    ipv6_disabled: bool,
}

impl DnsCryptModule {
    pub fn new(logger: Arc<Mutex<Logger>>) -> Self {
        let mut module = Self {
            enabled: false,
            servers: Vec::new(),
            next_server_id: 1,
            logger,
            selected_server: None,
            new_server_name: String::new(),
            new_server_address: String::new(),
            new_server_provider: String::new(),
            edit_mode: false,
            connection_status: "未连接".to_string(),
            dns_leak_protection: true,
            ipv6_disabled: false,
        };
        
        // 添加一些示例服务器
        module.add_example_servers();
        
        // 记录模块初始化日志
        if let Ok(mut logger) = module.logger.lock() {
            logger.info("DNSCrypt", "DNSCrypt模块已初始化");
        }
        
        module
    }
    
    // 添加示例服务器
    fn add_example_servers(&mut self) {
        // 添加一些示例DNSCrypt服务器
        let mut server1 = DnsCryptServer::new(
            self.next_server_id,
            "Cloudflare",
            "1.1.1.1:443",
            "2.dnscrypt-cert.cloudflare"
        );
        server1.description = "Cloudflare的DNS服务，注重隐私保护".to_string();
        server1.dnssec = true;
        server1.no_logs = true;
        self.servers.push(server1);
        self.next_server_id += 1;
        
        let mut server2 = DnsCryptServer::new(
            self.next_server_id,
            "Google",
            "8.8.8.8:443",
            "2.dnscrypt-cert.google"
        );
        server2.description = "Google的公共DNS服务".to_string();
        server2.dnssec = true;
        server2.no_logs = false;
        self.servers.push(server2);
        self.next_server_id += 1;
        
        let mut server3 = DnsCryptServer::new(
            self.next_server_id,
            "Quad9",
            "9.9.9.9:443",
            "2.dnscrypt-cert.quad9"
        );
        server3.description = "Quad9提供的安全DNS服务，可阻止恶意域名".to_string();
        server3.dnssec = true;
        server3.no_logs = true;
        self.servers.push(server3);
        self.next_server_id += 1;
    }
    
    // 添加新服务器
    fn add_server(&mut self, server: DnsCryptServer) {
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("DNSCrypt", &format!("添加新服务器: {}", server.name));
        }
        self.servers.push(server);
        self.next_server_id += 1;
    }
    
    // 删除服务器
    fn remove_server(&mut self, id: usize) {
        if let Some(index) = self.servers.iter().position(|s| s.id == id) {
            let server = &self.servers[index];
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("DNSCrypt", &format!("删除服务器: {}", server.name));
            }
            self.servers.remove(index);
            if self.selected_server == Some(id) {
                self.selected_server = None;
            }
        }
    }
    
    // 启用/禁用DNSCrypt
    fn toggle_dnscrypt(&mut self) {
        // 先获取当前状态的副本
        let new_enabled = !self.enabled;
        let status_message = if new_enabled { "启用" } else { "禁用" };
        
        // 记录日志
        {
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("DNSCrypt", &format!("DNSCrypt已{}", status_message));
            }
        }
        
        // 更新状态
        self.enabled = new_enabled;
        self.connection_status = if new_enabled { "正在连接..." } else { "未连接" }.to_string();
        
        // 在实际应用中，这里会启动或停止DNSCrypt服务
        if new_enabled {
            // 在实际应用中，这里会有异步连接逻辑
            self.connection_status = "已连接".to_string();
        }
    }
    
    // 启用/禁用服务器
    fn toggle_server(&mut self, id: usize) {
        // 先查找服务器并获取必要信息，避免同时借用
        let server_info = self.servers.iter_mut()
            .find(|s| s.id == id)
            .map(|server| {
                let name = server.name.clone();
                let new_state = !server.enabled;
                server.enabled = new_state;
                (name, new_state)
            });
        
        // 如果找到了服务器，记录日志
        if let Some((name, enabled)) = server_info {
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("DNSCrypt", &format!("服务器 '{}' 已{}", name, if enabled { "启用" } else { "禁用" }));
            }
        }
    }
    
    // 渲染UI
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("DNSCrypt").color(DNS_COLOR).strong());
            ui.add_space(10.0);
            
            let status_text = &self.connection_status;
            let status_color = match status_text.as_str() {
                "已连接" => Color32::GREEN,
                "正在连接..." => Color32::YELLOW,
                _ => Color32::RED,
            };
            ui.label(RichText::new(status_text).color(status_color).strong());
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(if self.enabled { "停止DNSCrypt" } else { "启动DNSCrypt" }).clicked() {
                    self.toggle_dnscrypt();
                }
            });
        });
        
        ui.separator();
        
        // DNSCrypt简介
        ui.collapsing("关于DNSCrypt", |ui| {
            ui.label("DNSCrypt是一种用于保护DNS查询的协议，可以防止DNS劫持和监听。");
            ui.label("通过加密DNS查询，DNSCrypt可以帮助您保护隐私并避免DNS泄露。");
            ui.label("官方网站: https://dnscrypt.info/");
        });
        
        // DNSCrypt设置
        ui.group(|ui| {
            ui.heading("DNSCrypt设置");
            
            ui.checkbox(&mut self.dns_leak_protection, "DNS泄露保护");
            ui.checkbox(&mut self.ipv6_disabled, "禁用IPv6解析");
        });
        
        ui.separator();
        
        // 服务器管理区域
        ui.horizontal(|ui| {
            ui.heading("DNSCrypt服务器");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("添加服务器").clicked() {
                    self.edit_mode = true;
                }
            });
        });
        
        // 服务器列表
        ScrollArea::vertical().show(ui, |ui| {
            Grid::new("dnscrypt_servers_grid")
                .num_columns(6)
                .striped(true)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    // 表头
                    ui.label(RichText::new("启用").strong());
                    ui.label(RichText::new("名称").strong());
                    ui.label(RichText::new("地址").strong());
                    ui.label(RichText::new("DNSSEC").strong());
                    ui.label(RichText::new("无日志").strong());
                    ui.label(RichText::new("操作").strong());
                    ui.end_row();
                    
                    // 服务器列表
                    let servers_copy = self.servers.clone();
                    for (_index, server) in servers_copy.iter().enumerate() {
                        // 启用/禁用复选框
                        let mut enabled = server.enabled;
                        if ui.checkbox(&mut enabled, "").changed() {
                            self.toggle_server(server.id);
                        }
                        
                        // 服务器名称
                        let server_text = RichText::new(&server.name);
                        if ui.selectable_label(self.selected_server == Some(server.id), server_text).clicked() {
                            self.selected_server = Some(server.id);
                        }
                        
                        // 服务器地址
                        ui.label(&server.address);
                        
                        // DNSSEC支持
                        ui.label(if server.dnssec { "✓" } else { "✗" });
                        
                        // 无日志政策
                        ui.label(if server.no_logs { "✓" } else { "✗" });
                        
                        // 操作按钮（修复借用冲突）
                        let server_id = server.id;
                        ui.horizontal(|ui| {
                            if ui.button("编辑").clicked() {
                                self.selected_server = Some(server_id);
                                self.edit_mode = true;
                            }
                            if ui.button("删除").clicked() {
                                self.remove_server(server_id);
                            }
                        });
                        
                        ui.end_row();
                    }
                });
        });
        
        // 服务器详情区域
        if let Some(server_id) = self.selected_server {
            if let Some(server) = self.servers.iter().find(|s| s.id == server_id) {
                ui.separator();
                ui.heading("服务器详情");
                
                Grid::new("server_details_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("名称:");
                        ui.label(&server.name);
                        ui.end_row();
                        
                        ui.label("地址:");
                        ui.label(&server.address);
                        ui.end_row();
                        
                        ui.label("提供商名称:");
                        ui.label(&server.provider_name);
                        ui.end_row();
                        
                        ui.label("DNSSEC支持:");
                        ui.label(if server.dnssec { "是" } else { "否" });
                        ui.end_row();
                        
                        ui.label("无日志政策:");
                        ui.label(if server.no_logs { "是" } else { "否" });
                        ui.end_row();
                        
                        ui.label("描述:");
                        ui.label(&server.description);
                        ui.end_row();
                    });
            }
        }
        
        // 添加/编辑服务器对话框
        if self.edit_mode {
            // 在实际应用中，这里会使用一个模态对话框
            // 简化起见，这里直接在主界面上显示编辑区域
            ui.separator();
            ui.heading(if self.selected_server.is_some() { "编辑服务器" } else { "添加服务器" });
            
            let mut server_name = self.new_server_name.clone();
            ui.horizontal(|ui| {
                ui.label("服务器名称:");
                if ui.text_edit_singleline(&mut server_name).changed() {
                    self.new_server_name = server_name;
                }
            });
            
            let mut server_address = self.new_server_address.clone();
            ui.horizontal(|ui| {
                ui.label("服务器地址:");
                if ui.text_edit_singleline(&mut server_address).changed() {
                    self.new_server_address = server_address;
                }
            });
            
            let mut server_provider = self.new_server_provider.clone();
            ui.horizontal(|ui| {
                ui.label("提供商名称:");
                if ui.text_edit_singleline(&mut server_provider).changed() {
                    self.new_server_provider = server_provider;
                }
            });
            
            ui.horizontal(|ui| {
                if ui.button("取消").clicked() {
                    self.edit_mode = false;
                    self.new_server_name.clear();
                    self.new_server_address.clear();
                    self.new_server_provider.clear();
                }
                
                if ui.button("保存").clicked() {
                    // 保存服务器逻辑
                    if !self.new_server_name.is_empty() && !self.new_server_address.is_empty() && !self.new_server_provider.is_empty() {
                        let new_server = DnsCryptServer::new(
                            self.next_server_id,
                            &self.new_server_name,
                            &self.new_server_address,
                            &self.new_server_provider
                        );
                        self.add_server(new_server);
                        self.new_server_name.clear();
                        self.new_server_address.clear();
                        self.new_server_provider.clear();
                        self.edit_mode = false;
                    }
                }
            });
        }
    }
}