use eframe::egui::{self, Color32, RichText, Ui};
use std::sync::{Arc, Mutex};

// 导入各个模块
use crate::firewall::FirewallModule;
use crate::tor::TorModule;
use crate::dnscrypt::DnsCryptModule;
use crate::i2p::I2PModule;
use crate::proxy::ProxyModule;
use crate::logger::Logger;

// 定义模块颜色
pub const TOR_COLOR: Color32 = Color32::from_rgb(89, 49, 107); // #59316B
pub const DNS_COLOR: Color32 = Color32::from_rgb(0, 92, 185);  // 蓝色
pub const I2P_COLOR: Color32 = Color32::from_rgb(102, 51, 153); // 紫色
pub const FIREWALL_COLOR: Color32 = Color32::from_rgb(220, 53, 69); // 红色
pub const SETTINGS_COLOR: Color32 = Color32::from_rgb(108, 117, 125); // 灰色
pub const LOG_COLOR: Color32 = Color32::from_rgb(108, 117, 125); // 灰色

// 定义应用程序的标签页
#[derive(PartialEq)]
enum Tab {
    Tor,
    DnsCrypt,
    I2P,
    Firewall,
    Proxy,
    Logs,
    Settings,
}

// 主应用程序结构
pub struct InviZibleApp {
    current_tab: Tab,
    tor_module: TorModule,
    dnscrypt_module: DnsCryptModule,
    i2p_module: I2PModule,
    firewall_module: FirewallModule,
    proxy_module: ProxyModule,
    logger: Arc<Mutex<Logger>>,
}

impl InviZibleApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 设置默认字体和样式
        let mut style = (*cc.egui_ctx.style()).clone();
        style.text_styles = egui::TextStyle::default_text_styles();
        cc.egui_ctx.set_style(style);
        
        // 创建日志记录器
        let logger = Arc::new(Mutex::new(Logger::new()));
        
        // 创建应用程序实例
        Self {
            current_tab: Tab::Tor,
            tor_module: TorModule::new(Arc::clone(&logger)),
            dnscrypt_module: DnsCryptModule::new(Arc::clone(&logger)),
            i2p_module: I2PModule::new(Arc::clone(&logger)),
            firewall_module: FirewallModule::new(Arc::clone(&logger)),
            proxy_module: ProxyModule::new(Arc::clone(&logger)),
            logger,
        }
    }
    
    // 渲染顶部导航栏
    fn render_top_panel(&mut self, ui: &mut Ui) {
        egui::TopBottomPanel::top("top_panel").show_inside(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 10.0;
                
                self.tab_button(ui, Tab::Tor, "Tor", TOR_COLOR);
                self.tab_button(ui, Tab::DnsCrypt, "DNSCrypt", DNS_COLOR);
                self.tab_button(ui, Tab::I2P, "I2P", I2P_COLOR);
                self.tab_button(ui, Tab::Firewall, "防火墙", FIREWALL_COLOR);
                self.tab_button(ui, Tab::Proxy, "代理", SETTINGS_COLOR);
                self.tab_button(ui, Tab::Logs, "日志", LOG_COLOR);
                self.tab_button(ui, Tab::Settings, "设置", SETTINGS_COLOR);
            });
        });
    }
    
    // 创建标签页按钮
    fn tab_button(&mut self, ui: &mut Ui, tab: Tab, name: &str, color: Color32) {
        let selected = self.current_tab == tab;
        let text = RichText::new(name).color(if selected { color } else { Color32::GRAY });
        let button = egui::Button::new(text).selected(selected);
        
        if ui.add(button).clicked() {
            self.current_tab = tab;
        }
    }
    
    // 渲染当前选中的标签页内容
    fn render_current_tab(&mut self, ui: &mut Ui) {
        match self.current_tab {
            Tab::Tor => self.tor_module.ui(ui),
            Tab::DnsCrypt => self.dnscrypt_module.ui(ui),
            Tab::I2P => self.i2p_module.ui(ui),
            Tab::Firewall => self.firewall_module.ui(ui),
            Tab::Proxy => self.proxy_module.ui(ui),
            Tab::Logs => {
                if let Ok(logger) = self.logger.lock() {
                    logger.ui(ui);
                }
            },
            Tab::Settings => {
                ui.heading("设置");
                ui.separator();
                ui.label("全局设置选项将在这里显示");
                // 这里可以添加全局设置选项
            },
        }
    }
}

// 实现eframe应用程序特性
impl eframe::App for InviZibleApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.render_top_panel(ui);
            ui.separator();
            self.render_current_tab(ui);
        });
    }
}