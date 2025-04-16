use eframe::egui::{self, Color32, RichText, Ui, Grid, ScrollArea};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

use crate::logger::Logger;
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
    // 删除隧道方法保持原样
    fn remove_tunnel(&mut self, id: usize) {
        let tunnel_index = self.tunnels.iter().position(|t| t.id == id);
        if let Some(index) = tunnel_index {
            let tunnel_name = self.tunnels[index].name.clone();
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("I2P", &format!("删除隧道: {}", tunnel_name));
            }
            self.tunnels.remove(index);
            if self.selected_tunnel == Some(id) {
                self.selected_tunnel = None;
            }
        }
    }
    
    // 启用/禁用I2P
    fn toggle_i2p(&mut self) {
        // 先获取当前状态的副本，避免同时借用
        let new_enabled = !self.enabled;
        let status_message = if new_enabled { "启用" } else { "禁用" };
        
        // 记录日志
        {
            // 使用单独的作用域限制logger的借用范围
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("I2P", &format!("I2P已{}", status_message));
            }
        }
        
        // 更新状态
        self.enabled = new_enabled;
        self.connection_status = if new_enabled { "正在连接..." } else { "未连接" }.to_string();
        
        // 在实际应用中，这里会启动或停止I2P服务
        if new_enabled {
            // 在实际应用中，这里会有异步连接逻辑
            // 模拟连接成功
            self.connection_status = "已连接".to_string();
            // 模拟带宽数据
            self.bandwidth_in = 128;
            self.bandwidth_out = 64;
        } else {
            // 重置带宽数据
            self.bandwidth_in = 0;
            self.bandwidth_out = 0;
        }
    }
    
    // 打开I2P控制台
    fn open_i2p_console(&mut self) {
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("I2P", "正在打开I2P控制台");
        }
        
        // 在实际应用中，这里会打开I2P控制台网页
        // 例如使用webbrowser库打开http://127.0.0.1:7657/
        if let Err(e) = std::process::Command::new("cmd")
            .args(["/c", "start", "http://127.0.0.1:7657/"])
            .spawn() {
            if let Ok(mut logger) = self.logger.lock() {
                logger.error("I2P", &format!("无法打开I2P控制台: {}", e));
            }
        }
    }
    
    // 将for循环移到UI方法内的正确位置
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
                    
                    // 修改后的隧道列表循环
                    // 先收集所有需要的隧道信息，避免在循环中借用self
                    let tunnels_info: Vec<_> = self.tunnels.iter().map(|tunnel| {
                        (
                            tunnel.id,
                            tunnel.enabled,
                            tunnel.name.clone(),
                            tunnel.tunnel_type.clone(),
                            tunnel.local_port,
                            self.selected_tunnel == Some(tunnel.id)
                        )
                    }).collect();
                    
                    for (tunnel_id, mut enabled, tunnel_name, tunnel_type, local_port, is_selected) in tunnels_info {
                        // 启用/禁用复选框
                        if ui.checkbox(&mut enabled, "")
                            .on_hover_text("启用/禁用该隧道")
                            .changed() {
                            // 在实际应用中，这里应该更新隧道的启用状态
                            if let Some(tunnel) = self.tunnels.iter_mut().find(|t| t.id == tunnel_id) {
                                tunnel.enabled = enabled;
                            }
                        }
                        
                        // 隧道名称选择
                        if ui.selectable_label(is_selected, &tunnel_name).clicked() {
                            self.selected_tunnel = Some(tunnel_id);
                        }
                        
                        // 隧道类型
                        let type_text = match tunnel_type {
                            TunnelType::Client => "客户端",
                            TunnelType::Server => "服务端",
                        };
                        ui.label(type_text);
                        
                        // 本地端口
                        ui.label(local_port.to_string());
                        
                        // 操作按钮
                        let tunnel_id_copy = tunnel_id; // 创建一个副本用于闭包
                        ui.horizontal(|ui| {
                            if ui.button("编辑").clicked() {
                                self.selected_tunnel = Some(tunnel_id_copy);
                                self.edit_mode = true;
                            }
                            if ui.button("删除").clicked() {
                                self.remove_tunnel(tunnel_id_copy);
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
            // 提前获取所需数据，避免在闭包中直接借用self
            let is_edit_mode = self.edit_mode;
            let has_selected_tunnel = self.selected_tunnel.is_some();
            let window_title = if has_selected_tunnel { "编辑隧道" } else { "添加隧道" };
            
            // 创建可变引用的副本，以便在闭包中使用
            let mut new_tunnel_name = self.new_tunnel_name.clone();
            let mut new_tunnel_type = self.new_tunnel_type.clone();
            let mut new_tunnel_port = self.new_tunnel_port;
            let mut new_tunnel_destination = self.new_tunnel_destination.clone();
            let next_tunnel_id = self.next_tunnel_id;
            
            // 使用模态对话框进行隧道编辑
            let mut still_open = is_edit_mode;
            egui::Window::new(window_title)
                .open(&mut still_open)
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("隧道名称:");
                        ui.text_edit_singleline(&mut new_tunnel_name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("隧道类型:");
                        egui::ComboBox::from_id_source("tunnel_type_combo")
                            .selected_text(match new_tunnel_type {
                                TunnelType::Client => "客户端",
                                TunnelType::Server => "服务端",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut new_tunnel_type, TunnelType::Client, "客户端");
                                ui.selectable_value(&mut new_tunnel_type, TunnelType::Server, "服务端");
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("本地端口:");
                        let mut tunnel_port = new_tunnel_port.to_string();
                        if ui.text_edit_singleline(&mut tunnel_port).changed() {
                            if let Ok(port) = tunnel_port.parse::<u16>() {
                                new_tunnel_port = port;
                            }
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.label("目标地址:");
                        ui.text_edit_singleline(&mut new_tunnel_destination);
                    });

                    // 保存用户操作的结果
                    let mut save_clicked = false;
                    let mut cancel_clicked = false;
                    
                    ui.horizontal(|ui| {
                        if ui.button("取消").clicked() {
                            cancel_clicked = true;
                        }

                        if ui.button("保存").clicked() {
                            if !new_tunnel_name.is_empty() && !new_tunnel_destination.is_empty() && new_tunnel_port > 0 {
                                save_clicked = true;
                            }
                        }
                    });
                    
                    // 返回用户操作结果和表单数据
                    (save_clicked, cancel_clicked, new_tunnel_name, new_tunnel_type, new_tunnel_port, new_tunnel_destination)
                })
                .and_then(|inner_result| inner_result.inner)
                .map(|(save_clicked, cancel_clicked, name, tunnel_type, port, destination)| {
                    // 根据用户操作更新状态
                    if save_clicked {
                        let new_tunnel = I2PTunnel::new(
                            next_tunnel_id,
                            &name,
                            tunnel_type,
                            port,
                            &destination
                        );
                        self.add_tunnel(new_tunnel);
                        self.new_tunnel_name.clear();
                        self.new_tunnel_destination.clear();
                        self.new_tunnel_port = 0;
                        self.edit_mode = false;
                    } else if cancel_clicked {
                        self.edit_mode = false;
                        self.new_tunnel_name.clear();
                        self.new_tunnel_destination.clear();
                        self.new_tunnel_port = 0;
                    } else {
                        // 更新表单数据，但不关闭窗口
                        self.new_tunnel_name = name;
                        self.new_tunnel_type = tunnel_type;
                        self.new_tunnel_port = port;
                        self.new_tunnel_destination = destination;
                    }
                });
                
            // 如果窗口被关闭，更新edit_mode
            if !still_open {
                self.edit_mode = false;
            }
        }
    }
}