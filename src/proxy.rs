use eframe::egui::{self, Color32, RichText, Ui, Grid, ScrollArea};
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

use crate::logger::{Logger, LogLevel};
use crate::app::SETTINGS_COLOR;

// 代理协议类型
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProxyProtocol {
    HTTP,
    SOCKS5,
}

// 代理服务配置
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub enabled: bool,
    pub protocol: ProxyProtocol,
    pub listen_address: String,
    pub listen_port: u16,
    pub tor_enabled: bool,
    pub dnscrypt_enabled: bool,
    pub i2p_enabled: bool,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            protocol: ProxyProtocol::SOCKS5,
            listen_address: "127.0.0.1".to_string(),
            listen_port: 1080,
            tor_enabled: true,
            dnscrypt_enabled: true,
            i2p_enabled: true,
        }
    }
}

// 代理模块结构
pub struct ProxyModule {
    config: ProxyConfig,
    logger: Arc<Mutex<Logger>>,
    status: String,
    port_conflict: bool,
    port_checking: bool,
}

impl ProxyModule {
    pub fn new(logger: Arc<Mutex<Logger>>) -> Self {
        let module = Self {
            config: ProxyConfig::default(),
            logger,
            status: "未启动".to_string(),
            port_conflict: false,
            port_checking: false,
        };
        
        // 记录模块初始化日志
        if let Ok(mut logger) = module.logger.lock() {
            logger.info("代理", "代理模块已初始化");
        }
        
        module
    }
    
    // 启动代理服务
    fn start_proxy(&mut self) {
        if self.port_conflict {
            if let Ok(mut logger) = self.logger.lock() {
                logger.error("代理", "无法启动代理服务：端口冲突");
            }
            return;
        }
        
        self.config.enabled = true;
        self.status = "运行中".to_string();
        
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("代理", &format!("代理服务已启动 ({}:{})", self.config.listen_address, self.config.listen_port));
        }
        
        // 在实际应用中，这里会启动代理服务器
    }
    
    // 停止代理服务
    fn stop_proxy(&mut self) {
        self.config.enabled = false;
        self.status = "未启动".to_string();
        
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("代理", "代理服务已停止");
        }
        
        // 在实际应用中，这里会停止代理服务器
    }
    
    // 检查端口冲突
    fn check_port_conflict(&mut self) {
        self.port_checking = true;
        
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("代理", &format!("正在检查端口 {} 是否可用...", self.config.listen_port));
        }
        
        // 在实际应用中，这里会使用端口扫描器检查端口是否被占用
        // 模拟检查过程
        let port_in_use = false; // 假设端口未被占用
        
        self.port_conflict = port_in_use;
        self.port_checking = false;
        
        if let Ok(mut logger) = self.logger.lock() {
            if self.port_conflict {
                logger.warning("代理", &format!("端口 {} 已被占用", self.config.listen_port));
            } else {
                logger.info("代理", &format!("端口 {} 可用", self.config.listen_port));
            }
        }
    }
    
    // 切换代理协议
    fn toggle_protocol(&mut self) {
        self.config.protocol = match self.config.protocol {
            ProxyProtocol::HTTP => ProxyProtocol::SOCKS5,
            ProxyProtocol::SOCKS5 => ProxyProtocol::HTTP,
        };
        
        if let Ok(mut logger) = self.logger.lock() {
            logger.info("代理", &format!("代理协议已更改为 {:?}", self.config.protocol));
        }
        
        // 如果代理正在运行，需要重启服务
        if self.config.enabled {
            self.stop_proxy();
            self.start_proxy();
        }
    }
    
    // 渲染UI
    pub fn ui(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.heading(RichText::new("代理服务").color(SETTINGS_COLOR).strong());
            ui.add_space(10.0);
            
            let status_text = &self.status;
            let status_color = match status_text.as_str() {
                "运行中" => Color32::GREEN,
                _ => Color32::RED,
            };
            ui.label(RichText::new(status_text).color(status_color).strong());
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button(if self.config.enabled { "停止代理" } else { "启动代理" }).clicked() {
                    if self.config.enabled {
                        self.stop_proxy();
                    } else {
                        self.start_proxy();
                    }
                }
            });
        });
        
        ui.separator();
        
        // 代理简介
        ui.collapsing("关于代理服务", |ui| {
            ui.label("代理服务允许您通过统一的接口使用Tor、DNSCrypt和I2P功能。");
            ui.label("您可以配置应用程序使用此代理来保护网络流量和隐私。");
        });
        
        ui.separator();
        
        // 代理设置
        ui.heading("代理设置");
        
        Grid::new("proxy_settings_grid")
            .num_columns(2)
            .spacing([10.0, 8.0])
            .show(ui, |ui| {
                // 代理协议
                ui.label("代理协议:");
                ui.horizontal(|ui| {
                    let protocol_text = match self.config.protocol {
                        ProxyProtocol::HTTP => "HTTP",
                        ProxyProtocol::SOCKS5 => "SOCKS5",
                    };
                    if ui.selectable_label(true, protocol_text).clicked() {
                        self.toggle_protocol();
                    }
                });
                ui.end_row();
                
                // 监听地址
                ui.label("监听地址:");
                let mut listen_address = self.config.listen_address.clone();
                if ui.text_edit_singleline(&mut listen_address).changed() {
                    self.config.listen_address = listen_address;
                    if self.config.enabled {
                        // 如果代理正在运行，需要重启服务
                        self.stop_proxy();
                        self.start_proxy();
                    }
                }
                ui.end_row();
                
                // 监听端口
                ui.label("监听端口:");
                ui.horizontal(|ui| {
                    let mut port_str = self.config.listen_port.to_string();
                    let response = ui.text_edit_singleline(&mut port_str);
                    if response.changed() {
                        if let Ok(port) = port_str.parse::<u16>() {
                            if port != self.config.listen_port {
                                self.config.listen_port = port;
                                self.check_port_conflict();
                                
                                if self.config.enabled {
                                    // 如果代理正在运行，需要重启服务
                                    self.stop_proxy();
                                    if !self.port_conflict {
                                        self.start_proxy();
                                    }
                                }
                            }
                        }
                    }
                    
                    if ui.button("检查端口").clicked() {
                        self.check_port_conflict();
                    }
                    
                    if self.port_conflict {
                        ui.label(RichText::new("端口冲突！").color(Color32::RED));
                    } else if !self.port_checking {
                        ui.label(RichText::new("端口可用").color(Color32::GREEN));
                    }
                });
                ui.end_row();
            });
        
        ui.separator();
        
        // 代理服务选项
        ui.heading("代理服务选项");
        
        ui.checkbox(&mut self.config.tor_enabled, "通过代理启用Tor服务");
        ui.checkbox(&mut self.config.dnscrypt_enabled, "通过代理启用DNSCrypt服务");
        ui.checkbox(&mut self.config.i2p_enabled, "通过代理启用I2P服务");
        
        if self.config.enabled {
            ui.separator();
            
            // 代理使用说明
            ui.heading("代理使用说明");
            
            ui.label("您可以在应用程序中使用以下代理设置:");
            
            let proxy_url = match self.config.protocol {
                ProxyProtocol::HTTP => format!("http://{}:{}", self.config.listen_address, self.config.listen_port),
                ProxyProtocol::SOCKS5 => format!("socks5://{}:{}", self.config.listen_address, self.config.listen_port),
            };
            
            ui.horizontal(|ui| {
                ui.label("代理地址:");
                ui.monospace(&proxy_url);
                if ui.button("复制").clicked() {
                    // 在实际应用中，这里会将代理地址复制到剪贴板
                    if let Ok(mut logger) = self.logger.lock() {
                        logger.info("代理", "代理地址已复制到剪贴板");
                    }
                }
            });
        }
    }
}