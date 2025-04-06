use eframe::egui::{self, Color32, RichText, Ui, Grid, ScrollArea};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::logger::Logger;
use crate::app::FIREWALL_COLOR;

// 防火墙规则类型
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RuleType {
    Application,
    Port,
    Address,
}

// 防火墙规则动作
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum RuleAction {
    Allow,
    Block,
}

// 防火墙规则结构
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FirewallRule {
    pub id: usize,
    pub name: String,
    pub rule_type: RuleType,
    pub action: RuleAction,
    pub enabled: bool,
    pub application_path: Option<String>,  // 用于应用程序规则
    pub port: Option<u16>,                 // 用于端口规则
    pub protocol: Option<String>,          // TCP/UDP
    pub address: Option<String>,           // 用于地址规则
    pub description: String,
}

impl FirewallRule {
    pub fn new(id: usize, name: &str, rule_type: RuleType) -> Self {
        Self {
            id,
            name: name.to_string(),
            rule_type,
            action: RuleAction::Block,
            enabled: true,
            application_path: None,
            port: None,
            protocol: Some("TCP".to_string()),
            address: None,
            description: String::new(),
        }
    }
}

// 防火墙模块结构
pub struct FirewallModule {
    enabled: bool,
    rules: Vec<FirewallRule>,
    next_rule_id: usize,
    logger: Arc<Mutex<Logger>>,
    selected_rule: Option<usize>,
    new_rule_name: String,
    new_rule_type: RuleType,
    edit_mode: bool,
    running_applications: HashMap<String, bool>, // 应用程序路径 -> 是否允许联网
}

impl FirewallModule {
    pub fn new(logger: Arc<Mutex<Logger>>) -> Self {
        let mut module = Self {
            enabled: false,
            rules: Vec::new(),
            next_rule_id: 1,
            logger,
            selected_rule: None,
            new_rule_name: String::new(),
            new_rule_type: RuleType::Application,
            edit_mode: false,
            running_applications: HashMap::new(),
        };
        
        // 添加一些示例规则
        module.add_example_rules();
        
        // 记录模块初始化日志
        if let Ok(mut logger) = module.logger.lock() {
            logger.info("防火墙", "防火墙模块已初始化");
        }
        
        module
    }
    
    // 添加示例规则
    fn add_example_rules(&mut self) {
        // 应用程序规则示例
        let mut rule1 = FirewallRule::new(self.next_rule_id, "阻止示例应用", RuleType::Application);
        rule1.application_path = Some("C:\\Program Files\\Example\\example.exe".to_string());
        rule1.action = RuleAction::Block;
        rule1.description = "阻止示例应用程序访问网络".to_string();
        self.rules.push(rule1);
        self.next_rule_id += 1;
        
        // 端口规则示例
        let mut rule2 = FirewallRule::new(self.next_rule_id, "阻止远程桌面", RuleType::Port);
        rule2.port = Some(3389);
        rule2.protocol = Some("TCP".to_string());
        rule2.action = RuleAction::Block;
        rule2.description = "阻止远程桌面连接（TCP 3389端口）".to_string();
        self.rules.push(rule2);
        self.next_rule_id += 1;
        
        // 地址规则示例
        let mut rule3 = FirewallRule::new(self.next_rule_id, "阻止特定IP", RuleType::Address);
        rule3.address = Some("192.168.1.100".to_string());
        rule3.action = RuleAction::Block;
        rule3.description = "阻止与特定IP地址的所有连接".to_string();
        self.rules.push(rule3);
        self.next_rule_id += 1;
    }
    
    // 添加新规则
    fn add_rule(&mut self, rule: FirewallRule) {
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("防火墙", &format!("添加新规则: {}", rule.name));
        }
        self.rules.push(rule);
        self.next_rule_id += 1;
    }
    
    // 删除规则
    fn remove_rule(&mut self, id: usize) {
        if let Some(index) = self.rules.iter().position(|r| r.id == id) {
            let rule = &self.rules[index];
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("防火墙", &format!("删除规则: {}", rule.name));
            }
            self.rules.remove(index);
            if self.selected_rule == Some(id) {
                self.selected_rule = None;
            }
        }
    }
    
    // 启用/禁用防火墙
    fn toggle_firewall(&mut self) {
        self.enabled = !self.enabled;
        let is_enabled = self.enabled; // 先保存状态，避免后续借用冲突
        
        {
            // 使用单独的作用域限制logger的借用范围
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("防火墙", &format!("防火墙已{}", if is_enabled { "启用" } else { "禁用" }));
            }
        }
    }
    
    // 启用/禁用规则
    fn toggle_rule(&mut self, id: usize) {
        // 先查找规则并获取必要信息，避免同时借用
        let rule_info = self.rules.iter_mut()
            .find(|r| r.id == id)
            .map(|rule| {
                let name = rule.name.clone();
                let new_state = !rule.enabled;
                rule.enabled = new_state;
                (name, new_state)
            });
        
        // 如果找到了规则，记录日志
        if let Some((name, enabled)) = rule_info {
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("防火墙", &format!("规则 '{}' 已{}", name, if enabled { "启用" } else { "禁用" }));
            }
        }
    }
    
    // 更改规则动作
    fn toggle_rule_action(&mut self, id: usize) {
        // 先查找规则并获取必要信息，避免同时借用
        let rule_info = self.rules.iter_mut()
            .find(|r| r.id == id)
            .map(|rule| {
                let name = rule.name.clone();
                let new_action = match rule.action {
                    RuleAction::Allow => RuleAction::Block,
                    RuleAction::Block => RuleAction::Allow,
                };
                rule.action = new_action.clone();
                (name, new_action)
            });
        
        // 如果找到了规则，记录日志
        if let Some((name, action)) = rule_info {
            if let Ok(mut logger) = self.logger.lock() {
                logger.info("防火墙", &format!("规则 '{}' 动作已更改为 {:?}", name, action));
            }
        }
    }
    
    // 扫描运行中的应用程序
    fn scan_running_applications(&mut self) {
        // 在实际实现中，这里会使用Windows API扫描运行中的应用程序
        // 这里只是模拟一些示例数据
        self.running_applications.clear();
        self.running_applications.insert("C:\\Program Files\\Internet Explorer\\iexplore.exe".to_string(), true);
        self.running_applications.insert("C:\\Program Files\\Mozilla Firefox\\firefox.exe".to_string(), true);
        self.running_applications.insert("C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".to_string(), true);
        self.running_applications.insert("C:\\Windows\\System32\\svchost.exe".to_string(), true);
        
        // 获取应用程序数量，避免同时借用
        let app_count = self.running_applications.len();
        
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("防火墙", &format!("扫描到 {} 个运行中的应用程序", app_count));
        }
    }
    
    // 渲染UI
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("防火墙").color(FIREWALL_COLOR).strong());
            ui.add_space(10.0);
            
            let status_text = if self.enabled { "已启用" } else { "已禁用" };
            let status_color = if self.enabled { Color32::GREEN } else { Color32::RED };
            ui.label(RichText::new(status_text).color(status_color).strong());
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(if self.enabled { "禁用防火墙" } else { "启用防火墙" }).clicked() {
                    self.toggle_firewall();
                }
            });
        });
        
        ui.separator();
        
        // 防火墙简介
        ui.collapsing("关于防火墙", |ui| {
            ui.label("防火墙可以控制应用程序的网络访问权限，阻止未授权的连接，保护您的计算机免受网络威胁。");
            ui.label("您可以创建基于应用程序、端口或IP地址的规则来精确控制网络流量。");
        });
        
        ui.separator();
        
        // 规则管理区域
        ui.horizontal(|ui| {
            ui.heading("防火墙规则");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("扫描应用程序").clicked() {
                    self.scan_running_applications();
                }
                if ui.button("添加规则").clicked() {
                    self.edit_mode = true;
                }
            });
        });
        
        // 规则列表
        ScrollArea::vertical().show(ui, |ui| {
            Grid::new("firewall_rules_grid")
                .num_columns(5)
                .striped(true)
                .spacing([10.0, 4.0])
                .show(ui, |ui| {
                    // 表头
                    ui.label(RichText::new("启用").strong());
                    ui.label(RichText::new("名称").strong());
                    ui.label(RichText::new("类型").strong());
                    ui.label(RichText::new("动作").strong());
                    ui.label(RichText::new("操作").strong());
                    ui.end_row();
                    
                    // 规则列表
                    let rules_clone = self.rules.clone(); // 克隆规则列表以避免借用冲突
                    for rule in &rules_clone {
                        // 启用/禁用复选框
                        let mut enabled = rule.enabled;
                        let rule_id = rule.id; // 先获取ID避免借用冲突
                        if ui.checkbox(&mut enabled, "").changed() {
                            self.toggle_rule(rule_id);
                        }
                        
                        // 规则名称
                        let rule_text = RichText::new(&rule.name);
                        if ui.selectable_label(self.selected_rule == Some(rule.id), rule_text).clicked() {
                            self.selected_rule = Some(rule.id);
                        }
                        
                        // 规则类型
                        let type_text = match rule.rule_type {
                            RuleType::Application => "应用程序",
                            RuleType::Port => "端口",
                            RuleType::Address => "地址",
                        };
                        ui.label(type_text);
                        
                        // 规则动作
                        let action_text = match rule.action {
                            RuleAction::Allow => RichText::new("允许").color(Color32::GREEN),
                            RuleAction::Block => RichText::new("阻止").color(Color32::RED),
                        };
                        if ui.selectable_label(false, action_text).clicked() {
                            self.toggle_rule_action(rule_id);
                        }
                        
                        // 操作按钮
                        let rule_id = rule.id; // 再次获取ID避免闭包中的借用冲突
                        ui.horizontal(|ui| {
                            if ui.button("编辑").clicked() {
                                // 编辑规则逻辑
                                self.selected_rule = Some(rule_id);
                                self.edit_mode = true;
                            }
                            if ui.button("删除").clicked() {
                                self.remove_rule(rule_id);
                            }
                        });
                        
                        ui.end_row();
                    }
                });
        });
        
        // 规则详情区域
        if let Some(rule_id) = self.selected_rule {
            if let Some(rule) = self.rules.iter().find(|r| r.id == rule_id) {
                ui.separator();
                ui.heading("规则详情");
                
                Grid::new("rule_details_grid")
                    .num_columns(2)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("名称:");
                        ui.label(&rule.name);
                        ui.end_row();
                        
                        ui.label("类型:");
                        ui.label(match rule.rule_type {
                            RuleType::Application => "应用程序",
                            RuleType::Port => "端口",
                            RuleType::Address => "地址",
                        });
                        ui.end_row();
                        
                        ui.label("动作:");
                        ui.label(match rule.action {
                            RuleAction::Allow => "允许",
                            RuleAction::Block => "阻止",
                        });
                        ui.end_row();
                        
                        match rule.rule_type {
                            RuleType::Application => {
                                ui.label("应用程序路径:");
                                if let Some(path) = &rule.application_path {
                                    ui.label(path);
                                }
                                ui.end_row();
                            },
                            RuleType::Port => {
                                ui.label("端口:");
                                if let Some(port) = rule.port {
                                    ui.label(port.to_string());
                                }
                                ui.end_row();
                                
                                ui.label("协议:");
                                if let Some(protocol) = &rule.protocol {
                                    ui.label(protocol);
                                }
                                ui.end_row();
                            },
                            RuleType::Address => {
                                ui.label("IP地址:");
                                if let Some(address) = &rule.address {
                                    ui.label(address);
                                }
                                ui.end_row();
                            },
                        }
                        
                        ui.label("描述:");
                        ui.label(&rule.description);
                        ui.end_row();
                    });
            }
        }
        
        // 添加/编辑规则对话框
        if self.edit_mode {
            let mut dialog_open = true;
            egui::Window::new(if self.selected_rule.is_some() { "编辑规则" } else { "添加规则" })
                .open(&mut dialog_open)
                .show(ui.ctx(), |ui| {
                    // 对话框内容
                    if !dialog_open {
                        self.edit_mode = false;
                        self.new_rule_name.clear();
                    }
                });
            ui.separator();
            ui.heading(if self.selected_rule.is_some() { "编辑规则" } else { "添加规则" });
            
            let mut rule_name = self.new_rule_name.clone();
            ui.horizontal(|ui| {
                ui.label("规则名称:");
                if ui.text_edit_singleline(&mut rule_name).changed() {
                    self.new_rule_name = rule_name;
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("规则类型:");
                egui::ComboBox::from_label("").selected_text(match self.new_rule_type {
                    RuleType::Application => "应用程序",
                    RuleType::Port => "端口",
                    RuleType::Address => "地址",
                }).show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.new_rule_type, RuleType::Application, "应用程序");
                    ui.selectable_value(&mut self.new_rule_type, RuleType::Port, "端口");
                    ui.selectable_value(&mut self.new_rule_type, RuleType::Address, "地址");
                });
            });

            match self.new_rule_type {
                RuleType::Application => {
                    ui.horizontal(|ui| {
                        ui.label("应用程序路径:");
                        if ui.text_edit_singleline(&mut self.new_rule_name).changed() {
                            // 自动填充规则名称
                            self.new_rule_name = self.new_rule_name.split("\\").last().unwrap_or("未知应用").to_string();
                        }
                    });
                },
                RuleType::Port => {
                    ui.horizontal(|ui| {
                        ui.label("端口号:");
                        ui.add(egui::DragValue::new(&mut self.new_rule_port).speed(1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("协议:");
                        egui::ComboBox::from_label("").selected_text("TCP").show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.new_rule_protocol, "TCP", "TCP");
                            ui.selectable_value(&mut self.new_rule_protocol, "UDP", "UDP");
                        });
                    });
                },
                RuleType::Address => {
                    ui.horizontal(|ui| {
                        ui.label("IP地址:");
                        ui.text_edit_singleline(&mut self.new_rule_address);
                    });
                },
            }

            ui.horizontal(|ui| {
                ui.label("动作:");
                egui::ComboBox::from_label("").selected_text(match self.new_rule_action {
                    RuleAction::Allow => "允许",
                    RuleAction::Block => "阻止",
                }).show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.new_rule_action, RuleAction::Allow, "允许");
                    ui.selectable_value(&mut self.new_rule_action, RuleAction::Block, "阻止");
                });
            });

            ui.horizontal(|ui| {
                ui.label("描述:");
                ui.text_edit_multiline(&mut self.new_rule_description);
            });
            
            ui.horizontal(|ui| {
                if ui.button("取消").clicked() {
                    self.edit_mode = false;
                    self.new_rule_name.clear();
                }
                
                if ui.button("保存").clicked() {
                    // 保存规则逻辑
                    if !self.new_rule_name.is_empty() {
                        let new_rule = FirewallRule::new(
                            self.next_rule_id,
                            &self.new_rule_name,
                            self.new_rule_type.clone()
                        );
                        self.add_rule(new_rule);
                        self.new_rule_name.clear();
                        self.edit_mode = false;
                    }
                }
            });
        }
        
        // 运行中的应用程序列表
        if !self.running_applications.is_empty() {
            ui.separator();
            ui.collapsing("运行中的应用程序", |ui| {
                Grid::new("running_apps_grid")
                    .num_columns(3)
                    .striped(true)
                    .spacing([10.0, 4.0])
                    .show(ui, |ui| {
                        // 表头
                        ui.label(RichText::new("应用程序路径").strong());
                        ui.label(RichText::new("网络访问").strong());
                        ui.label(RichText::new("操作").strong());
                        ui.end_row();
                        
                        // 克隆应用程序列表以避免借用冲突
                        let running_applications_clone = self.running_applications.clone();
                        // 应用程序列表
                        for (app_path, allowed) in &running_applications_clone {
                            ui.label(app_path);
                            
                            let status_text = if *allowed { RichText::new("允许").color(Color32::GREEN) } else { RichText::new("阻止").color(Color32::RED) };
                            ui.label(status_text);
                            
                            // 克隆数据以在闭包中使用
                            let app_path_clone = app_path.clone();
                            let allowed_clone = *allowed;
                            let next_rule_id = self.next_rule_id;
                            
                            ui.horizontal(|ui| {
                                if ui.button(if allowed_clone { "阻止" } else { "允许" }).clicked() {
                                    if let Some(allowed_mut) = self.running_applications.get_mut(&app_path_clone) {
                                        *allowed_mut = !allowed_clone;
                                        if let Ok(mut logger) = self.logger.lock() {
                                            logger.info("防火墙", &format!("{} 的网络访问已更改为 {}", app_path_clone, if *allowed_mut { "允许" } else { "阻止" }));
                                        }
                                    }
                                    if let Some(allowed_mut) = self.running_applications.get_mut(&app_path_clone) {
                                        *allowed_mut = !allowed_clone;
                                    }
                                }
                                
                                if ui.button("添加规则").clicked() {
                                    // 为该应用程序创建新规则
                                    let mut new_rule = FirewallRule::new(
                                        next_rule_id,
                                        &app_path_clone.split("\\").last().unwrap_or("未知应用"),
                                        RuleType::Application
                                    );
                                    new_rule.application_path = Some(app_path_clone);
                                    new_rule.action = if allowed_clone { RuleAction::Allow } else { RuleAction::Block };
                                    self.add_rule(new_rule);
                                }
                            });
                            
                            ui.end_row();
                        }
                    });
            });
        }
    }
}