use eframe::egui::{self, Color32, RichText, Ui, Grid, ScrollArea};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

use crate::logger::{Logger, LogLevel};
use crate::app::I2P_COLOR;

// I2P隧道类型
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TunnelType {
    Client,
    Server,
}

// I2P隧道结构
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct I2PTunnel {
    pub id: usize,
    pub name: String,
    pub tunnel_type: TunnelType,
    pub local_port: u16,
    pub destination: String,
    pub enabled: bool,
    pub description: String,
}

impl I2PTunnel {
    pub fn new(id: usize, name: &str, tunnel_type: TunnelType, local_port: u16, destination: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            tunnel_type,
            local_port,
            destination: destination.to_string(),
            enabled: true,
            description: String::new(),
        }
    }
}

// I2P模块结构
pub struct I2PModule {
    enabled: bool,
    tunnels: Vec<I2PTunnel>,
    next_tunnel_id: usize,
    logger: Arc<Mutex<Logger>>,
    selected_tunnel: Option<usize>,
    new_tunnel_name: String,
    new_tunnel_type: TunnelType,
    new_tunnel_port: u16,
    new_tunnel_destination: String,
    edit_mode: bool,
    connection_status: String,
    bandwidth_in: u32,  // KB/s
    bandwidth_out: u32, // KB/s
}

impl I2PModule {
    pub fn new(logger: Arc<Mutex<Logger>>) -> Self {
        let mut module = Self {
            enabled: false,
            tunnels: Vec::new(),
            next_tunnel_id: 1,
            logger,
            selected_tunnel: None,
            new_tunnel_name: String::new(),
            new_tunnel_type: TunnelType::Client,
            new_tunnel_port: 0,
            new_tunnel_destination: String::new(),
            edit_mode: false,
            connection_status: "未连接".to_string(),
            bandwidth_in: 0,
            bandwidth_out: 0,
        };
        
        // 添加一些示例隧道
        module.add_example_tunnels();
        
        // 记录模块初始化日志
        if let Ok(mut logger) = module.logger.lock() {
            logger.info("I2P", "I2P模块已初始化");
        }
        
        module
    }
    
    // 添加示例隧道
    fn add_example_tunnels(&mut self) {
        // 添加一些示例I2P隧道
        let mut tunnel1 = I2PTunnel::new(
            self.next_tunnel_id,
            "HTTP代理",
            TunnelType::Client,
            4444,
            "http://i2p-projekt.i2p"
        );
        tunnel1.description = "I2P HTTP代理隧道".to_string();
        self.tunnels.push(tunnel1);
        self.next_tunnel_id += 1;
        
        let mut tunnel2 = I2PTunnel::new(
            self.next_tunnel_id,
            "SOCKS代理",
            TunnelType::Client,
            4447,
            "socks://localhost:4447"
        );
        tunnel2.description = "I2P SOCKS代理隧道".to_string();
        self.tunnels.push(tunnel2);
        self.next_tunnel_id += 1;
        
        let mut tunnel3 = I2PTunnel::new(
            self.next_tunnel_id,
            "IRC服务器",
            TunnelType::Client,
            6668,
            "irc://irc.postman.i2p"
        );
        tunnel3.description = "连接到I2P IRC服务器的隧道".to_string();
        self.tunnels.push(tunnel3);
        self.next_tunnel_id += 1;
    }
    
    // 添加新隧道
    fn add_tunnel(&mut self, tunnel: I2PTunnel) {
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("I2P", &format!("添加新隧道: {}", tunnel.name));
        }
        self.tunnels.push(tunnel);
        self.next_tunnel_id += 1;
    }
    
    // 删除隧道
    fn remove_tunnel(&mut self, id: usize) {
        if let Some(index) = self.tunnels.iter().position(|t| t.id == id) {
            let tunnel = &self.tunnels[index];
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("I2P", &format!("删除隧道: {}", tunnel.name));
            }
            self.tunnels.remove(index);
            if self.selected_tunnel == Some(id) {
                self.selected_tunnel = None;
            }
        }
    }
    
    // 启用/禁用I2P
    fn toggle_i2p(&mut self) {
        self.enabled = !self.enabled;
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("I2P", &format!("I2P已{}", if self.enabled { "启用" } else { "禁用" }));
        }
        
        // 更新连接状态
        self.connection_status = if self.enabled { "正在连接..." } else { "未连接" }.to_string();
        
        // 在实际应用中，这里会启动或停止I2P服务
        // 模拟连接过程
        if self.enabled {
            // 在实际应用中，这里会有异步连接逻辑
            self.connection_status = "已连接".to_string();
            // 模拟带宽数据
            self.bandwidth_in = 128;
            self.bandwidth_out = 64;
        } else {
            self.bandwidth_in = 0;
            self.bandwidth_out = 0;
        }
    }
    
    // 启用/禁用隧道
    fn toggle_tunnel(&mut self, id: usize) {
        if let Some(tunnel) = self.tunnels.iter_mut().find(|t| t.id == id) {
            tunnel.enabled = !tunnel.enabled;
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("I2P", &format!("隧道 '{}' 已{}", tunnel.name, if tunnel.enabled { "启用" } else { "禁用" }));
            }
        }
    }
    
    // 打开I2P控制台
    fn open_i2p_console(&self) {
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("I2P", "打开I2P控制台");
        }
        
        // 在实际应用中，这里会使用系统默认浏览器打开I2P控制台
        // 例如：http://127.0.0.1:7657/
    }
    
    // 渲染UI
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("I2P网络").color(I2P_COLOR).strong());
            ui.add_space(10.0);
            
            let status_text = &self.connection_status;
            let status_color = match status_text.as_str() {
                "已连接" => Color32::GREEN,
                "正在连接..." => Color32::YELLOW,
                _ => Color32::RED,
            };
            ui.label(RichText::new(status_text).color(status_color).strong());
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(if self.enabled { "停止I2P" } else { "启动I2P" }).clicked() {
                    self.toggle_i2p();
                }
            });
        });
        
        ui.separator();
        
        // I2P简介
        ui.collapsing("关于I2P", |ui| {
            ui.label("I2P（Invisible Internet Project）是一个匿名网络层，允许进行抗审查和私密的通信。");
            ui.label("与Tor不同，I2P主要设计用于网络内部的通信，而不是访问外部互联网。");
            ui.label("官方网站: https://geti2p.net/");
            
            if ui.button("打开I2P控制台").clicked() {
                self.open_i2p_console();
            }
        });
        
        // 如果I2P已启用，显示带宽信息
        if self.enabled {
            ui.group(|ui| {
                ui.heading("带宽使用情况");
                
                ui.horizontal(|ui| {
                    ui.label("入站:");
                    ui.label(format!("{} KB/s", self.bandwidth_in));
                });
                
                ui.horizontal(|ui| {
                    ui.label("出站:");
                    ui.label(format!("{} KB/s", self.bandwidth_out));
                });
            });
        }
        
        ui.separator();
        
        // 隧道管理区域
        ui.horizontal(|ui| {
            ui.heading("I2P隧道");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("添加隧道").clicked() {
                    self.edit_mode = true;
                }
            });
        });
        
        // 隧道列表
        ScrollArea::vertical().show(ui, |ui| {
            Grid::new("i2p_tunnels_grid")
                .num_columns(5)
                .striped(true)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    // 表头
                    ui.label(RichText::new("启用").strong());
                    ui.label(RichText::new("名称").strong());
                    ui.label(RichText::new("类型").strong());
                    ui.label(RichText::new("本地端口").strong());
                    ui.label(RichText::new("操作").strong());
                    ui.end_row();
                    
                    // 隧道列表
                    for tunnel in &self.tunnels {
                        // 启用/禁用复选框
                        let mut enabled = tunnel.enabled;
                        if ui.checkbox(&mut enabled, "").changed() {
                            self.toggle_tunnel(tunnel.id);
                        }
                        
                        // 隧道名称
                        let tunnel_text = RichText::new(&tunnel.name);
                        if ui.selectable_label(self.selected_tunnel == Some(tunnel.id), tunnel_text).clicked() {
                            self.selected_tunnel = Some(tunnel.id);
                        }
                        
                        // 隧道类型
                        let type_text = match tunnel.tunnel_type {
                            TunnelType::Client => "客户端",
                            TunnelType::Server => "服务端",
                        };
                        ui.label(type_text);
                        
                        // 本地端口
                        ui.label(tunnel.local_port.to_string());
                        
                        // 操作按钮
                        ui.horizontal(|ui| {
                            if ui.button("编辑").clicked() {
                                // 编辑隧道逻辑
                                self.selected_tunnel = Some(tunnel.id);
                                self.edit_mode = true;
                            }
                            if ui.button("删除").clicked() {
                                self.remove_tunnel(tunnel.id);
                            }
                        });
                        
                        ui.end_row();
                    }
                });
        });
        
        // 隧道详情区域
        if let Some(tunnel_id) = self.selected_tunnel {
            if let Some(tunnel) = self.tunnels.iter().find(|t| t.id == tunnel_id) {
                ui.separator();
                ui.heading("隧道详情");
                
                Grid::new("tunnel_details_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("名称:");
                        ui.label(&tunnel.name);
                        ui.end_row();
                        
                        ui.label("类型:");
                        ui.label(match tunnel.tunnel_type {
                            TunnelType::Client => "客户端",
                            TunnelType::Server => "服务端",
                        });
                        ui.end_row();
                        
                        ui.label("本地端口:");
                        ui.label(tunnel.local_port.to_string());
                        ui.end_row();
                        
                        ui.label("目标地址:");
                        ui.label(&tunnel.destination);
                        ui.end_row();
                        
                        ui.label("描述:");
                        ui.label(&tunnel.description);
                        ui.end_row();
                    });
            }
        }
        
        // 添加/编辑隧道对话框
        if self.edit_mode {
            // 在实际应用中，这里会使用一个模态对话框
            // 简化起见，这里直接在主界面上显示编辑区域
            ui.separator();
            ui.heading(if self.selected_tunnel.is_some() { "编辑隧道" } else { "添加隧道" });
            
            let mut tunnel_name = self.new_tunnel_name.clone();
            ui.horizontal(|ui| {
                ui.label("隧道名称:");
                if ui.text_edit_singleline(&mut tunnel_name).changed() {
                    self.new_tunnel_name = tunnel_name;
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("隧道类型:");
                egui::ComboBox::from_id_source("tunnel_type_combo")
                    .selected_text(match self.new_tunnel_type {
                        TunnelType::Client => "客户端",
                        TunnelType::Server => "服务端",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.new_tunnel_type, TunnelType::Client, "客户端");
                        ui.selectable_value(&mut self.new_tunnel_type, TunnelType::Server, "服务端");
                    });
            });
            
            let mut tunnel_port = self.new_tunnel_port.to_string();
            ui.horizontal(|ui| {
                ui.label("本地端口:");
                if ui.text_edit_singleline(&mut tunnel_port).changed() {
                    if let Ok(port) = tunnel_port.parse::<u16>() {
                        self.new_tunnel_port = port;
                    }
                }
            });
            
            let mut tunnel_destination = self.new_tunnel_destination.clone();
            ui.horizontal(|ui| {
                ui.label("目标地址:");
                if ui.text_edit_singleline(&mut tunnel_destination).changed() {
                    self.new_tunnel_destination = tunnel_destination;
                }
            });
            
            ui.horizontal(|ui| {
                if ui.button("取消").clicked() {
                    self.edit_mode = false;
                    self.new_tunnel_name.clear();
                    self.new_tunnel_destination.clear();
                    self.new_tunnel_port = 0;
                }
                
                if ui.button("保存").clicked() {
                    // 保存隧道逻辑
                    if !self.new_tunnel_name.is_empty() && !self.new_tunnel_destination.is_empty() && self.new_tunnel_port > 0 {
                        let new_tunnel = I2PTunnel::new(
                            self.next_tunnel_id,
                            &self.new_tunnel_name,
                            self.new_tunnel_type.clone(),
                            self.new_tunnel_port,
                            &self.new_tunnel_destination
                        );
                        self.add_tunnel(new_tunnel);
                        self.new_tunnel_name.clear();
                        self.new_tunnel_destination.clear();
                        self.new_tunnel_port = 0;
                        self.edit_mode = false;
                    }
                }
            });
        }
    }
}