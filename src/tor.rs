use eframe::egui::{self, Color32, RichText, Ui, Grid, ScrollArea};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use std::process::Child;
use torut::control::TorControlConnection;
use tokio::runtime::Runtime;

use crate::logger::Logger;
use crate::app::TOR_COLOR;

// Tor网桥类型
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BridgeType {
    Vanilla,
    Obfs4,
    Snowflake,
    Meek,
}

// Tor网桥结构
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TorBridge {
    pub id: usize,
    pub name: String,
    pub bridge_type: BridgeType,
    pub address: String,
    pub enabled: bool,
}

impl TorBridge {
    pub fn new(id: usize, name: &str, bridge_type: BridgeType, address: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            bridge_type,
            address: address.to_string(),
            enabled: true,
        }
    }
}

// Tor节点类型
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeType {
    Relay,  // 中继节点
    Exit,   // 出口节点
}

// Tor模块结构
pub struct TorModule {
    enabled: bool,
    bridges: Vec<TorBridge>,
    next_bridge_id: usize,
    logger: Arc<Mutex<Logger>>,
    selected_bridge: Option<usize>,
    new_bridge_name: String,
    new_bridge_type: BridgeType,
    new_bridge_address: String,
    edit_mode: bool,
    run_as_node: bool,
    node_type: NodeType,
    connection_status: String,
    bandwidth_limit: u32,  // KB/s
    tor_process: Option<Child>
}

impl TorModule {
    pub fn new(logger: Arc<Mutex<Logger>>) -> Self {
        let mut module = Self {
            enabled: false,
            bridges: Vec::new(),
            next_bridge_id: 1,
            logger,
            selected_bridge: None,
            new_bridge_name: String::new(),
            new_bridge_type: BridgeType::Vanilla,
            new_bridge_address: String::new(),
            edit_mode: false,
            run_as_node: false,
            node_type: NodeType::Relay,
            connection_status: "未连接".to_string(),
            bandwidth_limit: 1024,  // 默认1MB/s
            tor_process: None,
        };
        
        // 添加一些示例网桥
        module.add_example_bridges();
        
        // 记录模块初始化日志
        if let Ok(mut logger) = module.logger.lock() {
            logger.info("Tor", "Tor模块已初始化");
        }
        
        module
    }
    
    // 添加示例网桥
    fn add_example_bridges(&mut self) {
        // 添加一些示例网桥
        let bridge1 = TorBridge::new(
            self.next_bridge_id,
            "Vanilla Bridge 1",
            BridgeType::Vanilla,
            "192.0.2.1:9001 A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0"
        );
        self.bridges.push(bridge1);
        self.next_bridge_id += 1;
        
        let bridge2 = TorBridge::new(
            self.next_bridge_id,
            "Obfs4 Bridge 1",
            BridgeType::Obfs4,
            "obfs4 192.0.2.2:9001 A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0 cert=ABCDEF... iat-mode=0"
        );
        self.bridges.push(bridge2);
        self.next_bridge_id += 1;
        
        let bridge3 = TorBridge::new(
            self.next_bridge_id,
            "Snowflake Bridge",
            BridgeType::Snowflake,
            "snowflake 192.0.2.3:9001 A1B2C3D4E5F6G7H8I9J0K1L2M3N4O5P6Q7R8S9T0 fingerprint=ABCDEF..."
        );
        self.bridges.push(bridge3);
        self.next_bridge_id += 1;
    }
    
    // 添加新网桥
    fn add_bridge(&mut self, bridge: TorBridge) {
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("Tor", &format!("添加新网桥: {}", bridge.name));
        }
        self.bridges.push(bridge);
        self.next_bridge_id += 1;
    }
    
    // 删除网桥
    fn remove_bridge(&mut self, id: usize) {
        if let Some(index) = self.bridges.iter().position(|b| b.id == id) {
            let bridge = &self.bridges[index];
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("Tor", &format!("删除网桥: {}", bridge.name));
            }
            self.bridges.remove(index);
            if self.selected_bridge == Some(id) {
                self.selected_bridge = None;
            }
        }
    }
    
    // 启用/禁用Tor
    async fn connect_to_tor(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let tor_control_port = 9051;
        let tor_control = TorControlConnection::connect("127.0.0.1", tor_control_port).await?;
        tor_control.authenticate("").await?;
        self.connection_status = "已连接".to_string();
        Ok(())
    }
    
    // 启用/禁用Tor
    fn toggle_tor(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // 先获取当前状态的副本，避免同时借用
        let new_enabled = !self.enabled;
        let status_message = if new_enabled { "启用" } else { "禁用" };
        
        // 记录日志
        {
            // 使用单独的作用域限制logger的借用范围
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("Tor", &format!("Tor已{}", status_message));
            }
        }
        
        // 更新状态
        self.enabled = new_enabled;
        self.connection_status = if new_enabled { "正在连接..." } else { "未连接" }.to_string();
        
        // 启动或停止Tor服务
        let tor_process = if new_enabled {
            Some(std::process::Command::new("tor")
                .arg("--RunAsDaemon")
                .arg("1")
                .spawn().expect("无法启动Tor进程"))
        } else {
            if let Some(mut process) = self.tor_process.take() {
                let _ = process.kill();
            }
            None
        };
        self.tor_process = tor_process;
        
        // 模拟连接过程
        if new_enabled {
            // 创建一个运行时来执行异步连接逻辑
            let rt = Runtime::new().unwrap();
            let result = rt.block_on(self.connect_to_tor());
            if let Err(e) = result {
                if let Ok(mut logger) = self.logger.lock() {
                    logger.error("Tor", &format!("连接Tor失败: {}", e));
                }
                self.connection_status = "连接失败".to_string();
            }
        }
        
        Ok(())
    }
    
    // 启用/禁用网桥
    fn toggle_bridge(&mut self, id: usize) {
        // 先查找网桥并获取必要信息，避免同时借用
        let bridge_info = self.bridges.iter_mut()
            .find(|b| b.id == id)
            .map(|bridge| {
                let name = bridge.name.clone();
                let new_state = !bridge.enabled;
                bridge.enabled = new_state;
                (name, new_state)
            });
        
        // 如果找到了网桥，记录日志
        if let Some((name, enabled)) = bridge_info {
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("Tor", &format!("网桥 '{}' 已{}", name, if enabled { "启用" } else { "禁用" }));
            }
        }
    }
    
    // 切换节点类型
    fn toggle_node_type(&mut self) {
        self.node_type = match self.node_type {
            NodeType::Relay => NodeType::Exit,
            NodeType::Exit => NodeType::Relay,
        };
        
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("Tor", &format!("节点类型已更改为 {:?}", self.node_type));
        }
    }
    
    // 打开Tor项目捐赠页面
    fn open_donation_page(&self) {
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("Tor", "打开Tor项目捐赠页面");
        }
        
        // 使用系统默认浏览器打开捐赠页面
        if let Err(e) = webbrowser::open("https://donate.torproject.org/") {
            if let Ok(mut logger) = self.logger.lock() {
                logger.error("Tor", &format!("无法打开捐赠页面: {}", e));
            }
        }
    }
    
    // 获取当前连接状态的副本
    fn get_connection_status(&self) -> String {
        self.connection_status.clone()
    }
    
    // 渲染UI
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("Tor洋葱网络").color(TOR_COLOR).strong());
            ui.add_space(10.0);
            
            let status_text = &self.connection_status;
            let status_color = match status_text.as_str() {
                "已连接" => Color32::GREEN,
                "正在连接..." => Color32::YELLOW,
                _ => Color32::RED,
            };
            ui.label(RichText::new(status_text).color(status_color).strong());
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(if self.enabled { "停止Tor" } else { "启动Tor" }).clicked() {
                    if let Err(e) = self.toggle_tor() {
                        if let Ok(mut logger) = self.logger.lock() {
                            logger.error("Tor", &format!("Tor操作失败: {}", e));
                        }
                    }
                }
            });
        });
        
        ui.separator();
        
        // Tor简介
        ui.collapsing("关于Tor", |ui| {
            ui.label("Tor是一个匿名通信网络，可以帮助您保护隐私和规避网络审查。");
            ui.label("通过Tor，您的网络流量会经过多个中继节点加密传输，使得第三方难以追踪您的真实位置和活动。");
            ui.label("官方网站: https://www.torproject.org/");
            
            ui.horizontal(|ui| {
                if ui.button("赞助Tor项目").clicked() {
                    self.open_donation_page();
                }
                
                ui.checkbox(&mut self.run_as_node, "运行节点服务来支持Tor");
            });
        });
        
        // 如果启用了节点服务，显示节点设置
        if self.run_as_node {
            ui.group(|ui| {
                ui.heading("节点服务设置");
                
                ui.horizontal(|ui| {
                    ui.label("节点类型:");
                    let node_type_text = match self.node_type {
                        NodeType::Relay => "中继节点",
                        NodeType::Exit => "出口节点",
                    };
                    if ui.selectable_label(true, node_type_text).clicked() {
                        if self.node_type == NodeType::Relay {
                            // 显示警告对话框
                            let response = egui::Window::new("警告")
                                .open(&mut true)
                                .show(ui.ctx(), |ui| {
                                    ui.label("运行出口节点可能会带来法律风险，因为其他用户的流量将通过您的网络连接离开Tor网络。");
                                    ui.horizontal(|ui| {
                                        if ui.button("确认").clicked() {
                                            true
                                        } else {
                                            false
                                        }
                                    })
                                });
                            if let Some(response) = response {
                                if response.inner {
                                    self.toggle_node_type();
                                }
                            }
                            ui.label(RichText::new("警告: 运行出口节点可能会带来法律风险，因为其他用户的流量将通过您的网络连接离开Tor网络。").color(Color32::RED));
                            // 如果用户确认，则切换节点类型
                            self.toggle_node_type();
                        } else {
                            self.toggle_node_type();
                        }
                    }
                });
                
                ui.horizontal(|ui| {
                    ui.label("带宽限制:");
                    ui.add(egui::Slider::new(&mut self.bandwidth_limit, 100..=10240).suffix(" KB/s"));
                });
            });
        }
        
        ui.separator();
        
        // 网桥管理区域
        ui.horizontal(|ui| {
            ui.heading("Tor网桥");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("添加网桥").clicked() {
                    self.edit_mode = true;
                }
            });
        });
        
        // 网桥列表
        ScrollArea::vertical().show(ui, |ui| {
            Grid::new("tor_bridges_grid")
                .num_columns(4)
                .striped(true)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    // 表头
                    ui.label(RichText::new("启用").strong());
                    ui.label(RichText::new("名称").strong());
                    ui.label(RichText::new("类型").strong());
                    ui.label(RichText::new("操作").strong());
                    ui.end_row();
                    
                    // 网桥列表
                    // 克隆网桥列表以避免借用冲突
                    let bridges_clone = self.bridges.clone();
                    for bridge in &bridges_clone {
                        // 启用/禁用复选框
                        let mut enabled = bridge.enabled;
                        let bridge_id = bridge.id; // 先获取ID避免借用冲突
                        if ui.checkbox(&mut enabled, "").changed() {
                            self.toggle_bridge(bridge_id);
                        }
                        
                        // 网桥名称
                        let bridge_text = RichText::new(&bridge.name);
                        if ui.selectable_label(self.selected_bridge == Some(bridge.id), bridge_text).clicked() {
                            self.selected_bridge = Some(bridge.id);
                        }
                        
                        // 网桥类型
                        let type_text = match bridge.bridge_type {
                            BridgeType::Vanilla => "Vanilla",
                            BridgeType::Obfs4 => "Obfs4",
                            BridgeType::Snowflake => "Snowflake",
                            BridgeType::Meek => "Meek",
                        };
                        ui.label(type_text);
                        
                        // 操作按钮
                        let bridge_id = bridge.id; // 再次获取ID避免闭包中的借用冲突
                        ui.horizontal(|ui| {
                            if ui.button("编辑").clicked() {
                                // 编辑网桥逻辑
                                self.selected_bridge = Some(bridge_id);
                                self.edit_mode = true;
                            }
                            if ui.button("删除").clicked() {
                                self.remove_bridge(bridge_id);
                            }
                        });
                        
                        ui.end_row();
                    }
                });
        });
        
        // 网桥详情区域
        if let Some(bridge_id) = self.selected_bridge {
            if let Some(bridge) = self.bridges.iter().find(|b| b.id == bridge_id) {
                ui.separator();
                ui.heading("网桥详情");
                
                Grid::new("bridge_details_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("名称:");
                        ui.label(&bridge.name);
                        ui.end_row();
                        
                        ui.label("类型:");
                        ui.label(match bridge.bridge_type {
                            BridgeType::Vanilla => "Vanilla",
                            BridgeType::Obfs4 => "Obfs4",
                            BridgeType::Snowflake => "Snowflake",
                            BridgeType::Meek => "Meek",
                        });
                        ui.end_row();
                        
                        ui.label("地址:");
                        ui.label(&bridge.address);
                        ui.end_row();
                    });
            }
        }
        
        // 添加/编辑网桥对话框
        if self.edit_mode {
            // 使用模态对话框编辑网桥
            let response = egui::Window::new(if self.selected_bridge.is_some() { "编辑网桥" } else { "添加网桥" })
                .open(&mut self.edit_mode)
                .show(ui.ctx(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("网桥名称:");
                        ui.text_edit_singleline(&mut self.new_bridge_name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("网桥类型:");
                        egui::ComboBox::from_id_source("bridge_type_combo")
                            .selected_text(match self.new_bridge_type {
                                BridgeType::Vanilla => "Vanilla",
                                BridgeType::Obfs4 => "Obfs4",
                                BridgeType::Snowflake => "Snowflake",
                                BridgeType::Meek => "Meek",
                            })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.new_bridge_type, BridgeType::Vanilla, "Vanilla");
                                ui.selectable_value(&mut self.new_bridge_type, BridgeType::Obfs4, "Obfs4");
                                ui.selectable_value(&mut self.new_bridge_type, BridgeType::Snowflake, "Snowflake");
                                ui.selectable_value(&mut self.new_bridge_type, BridgeType::Meek, "Meek");
                            });
                    });
                    ui.horizontal(|ui| {
                        ui.label("网桥地址:");
                        ui.text_edit_singleline(&mut self.new_bridge_address);
                    });
                    ui.horizontal(|ui| {
                        if ui.button("取消").clicked() {
                            false
                        } else if ui.button("保存").clicked() {
                            true
                        }
                    })
                });
            if let Some(response) = response {
                if response.inner {
                    if !self.new_bridge_name.is_empty() && !self.new_bridge_address.is_empty() {
                        let new_bridge = TorBridge::new(
                            self.next_bridge_id,
                            &self.new_bridge_name,
                            self.new_bridge_type.clone(),
                            &self.new_bridge_address
                        );
                        self.add_bridge(new_bridge);
                        self.new_bridge_name.clear();
                        self.new_bridge_address.clear();
                        self.edit_mode = false;
                    }
                }
            }
            ui.separator();
            ui.heading(if self.selected_bridge.is_some() { "编辑网桥" } else { "添加网桥" });
            
            let mut bridge_name = self.new_bridge_name.clone();
            ui.horizontal(|ui| {
                ui.label("网桥名称:");
                if ui.text_edit_singleline(&mut bridge_name).changed() {
                    self.new_bridge_name = bridge_name;
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("网桥类型:");
                egui::ComboBox::from_id_source("bridge_type_combo")
                    .selected_text(match self.new_bridge_type {
                        BridgeType::Vanilla => "Vanilla",
                        BridgeType::Obfs4 => "Obfs4",
                        BridgeType::Snowflake => "Snowflake",
                        BridgeType::Meek => "Meek",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.new_bridge_type, BridgeType::Vanilla, "Vanilla");
                        ui.selectable_value(&mut self.new_bridge_type, BridgeType::Obfs4, "Obfs4");
                        ui.selectable_value(&mut self.new_bridge_type, BridgeType::Snowflake, "Snowflake");
                        ui.selectable_value(&mut self.new_bridge_type, BridgeType::Meek, "Meek");
                    });
            });
            
            let mut bridge_address = self.new_bridge_address.clone();
            ui.horizontal(|ui| {
                ui.label("网桥地址:");
                if ui.text_edit_singleline(&mut bridge_address).changed() {
                    self.new_bridge_address = bridge_address;
                }
            });
            
            ui.horizontal(|ui| {
                if ui.button("取消").clicked() {
                    self.edit_mode = false;
                    self.new_bridge_name.clear();
                    self.new_bridge_address.clear();
                }
                
                if ui.button("保存").clicked() {
                    // 保存网桥逻辑
                    if !self.new_bridge_name.is_empty() && !self.new_bridge_address.is_empty() {
                        let new_bridge = TorBridge::new(
                            self.next_bridge_id,
                            &self.new_bridge_name,
                            self.new_bridge_type.clone(),
                            &self.new_bridge_address
                        );
                        self.add_bridge(new_bridge);
                        self.new_bridge_name.clear();
                        self.new_bridge_address.clear();
                        self.edit_mode = false;
                    }
                }
            });
        }
    }
}