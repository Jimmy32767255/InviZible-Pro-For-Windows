use eframe::egui;
use log::{info, LevelFilter};
use std::sync::{Arc, Mutex};

mod app;
mod firewall;
mod tor;
mod dnscrypt;
mod i2p;
mod proxy;
mod logger;
mod utils;

use app::InviZibleApp;

fn main() -> Result<(), eframe::Error> {
    // 初始化日志系统
    env_logger::Builder::new()
        .filter(None, LevelFilter::Info)
        .format_timestamp_secs()
        .init();
    
    info!("InviZible Pro for Windows 启动中...");
    
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(1000.0, 700.0)),
        min_window_size: Some(egui::vec2(800.0, 600.0)),
        icon_data: None, // 可以在这里添加应用图标
        ..Default::default()
    };
    
    // 启动GUI应用
    eframe::run_native(
        "InviZible Pro for Windows",
        options,
        Box::new(|cc| Box::new(InviZibleApp::new(cc)))
    )
}