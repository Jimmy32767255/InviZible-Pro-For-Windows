use eframe::egui::{self, Color32, RichText, ScrollArea, Ui};
use chrono::{DateTime, Local};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

// 日志级别枚举
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

// 日志条目结构
#[derive(Clone, Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<Local>,
    pub level: LogLevel,
    pub module: String,
    pub message: String,
}

impl LogEntry {
    pub fn new(level: LogLevel, module: &str, message: &str) -> Self {
        Self {
            timestamp: Local::now(),
            level,
            module: module.to_string(),
            message: message.to_string(),
        }
    }
    
    // 获取日志级别对应的颜色
    fn level_color(&self) -> Color32 {
        match self.level {
            LogLevel::Info => Color32::from_rgb(13, 110, 253),    // 蓝色
            LogLevel::Warning => Color32::from_rgb(255, 193, 7),  // 黄色
            LogLevel::Error => Color32::from_rgb(220, 53, 69),    // 红色
            LogLevel::Debug => Color32::from_rgb(108, 117, 125),  // 灰色
        }
    }
    
    // 获取日志级别的字符串表示
    fn level_str(&self) -> &'static str {
        match self.level {
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARN",
            LogLevel::Error => "ERROR",
            LogLevel::Debug => "DEBUG",
        }
    }
}

// 日志系统结构
pub struct Logger {
    logs: VecDeque<LogEntry>,
    max_logs: usize,
    filter_level: Option<LogLevel>,
    filter_module: Option<String>,
    auto_scroll: bool,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            logs: VecDeque::with_capacity(1000),
            max_logs: 1000,
            filter_level: None,
            filter_module: None,
            auto_scroll: true,
        }
    }
    
    // 添加日志条目
    pub fn log(&mut self, level: LogLevel, module: &str, message: &str) {
        let entry = LogEntry::new(level, module, message);
        self.logs.push_back(entry);
        
        // 如果超过最大日志数量，移除最旧的日志
        if self.logs.len() > self.max_logs {
            self.logs.pop_front();
        }
    }
    
    // 便捷日志方法
    pub fn info(&mut self, module: &str, message: &str) {
        self.log(LogLevel::Info, module, message);
    }
    
    pub fn warning(&mut self, module: &str, message: &str) {
        self.log(LogLevel::Warning, module, message);
    }
    
    pub fn error(&mut self, module: &str, message: &str) {
        self.log(LogLevel::Error, module, message);
    }
    
    pub fn debug(&mut self, module: &str, message: &str) {
        self.log(LogLevel::Debug, module, message);
    }
    
    // 清除所有日志
    pub fn clear(&mut self) {
        self.logs.clear();
    }
    
    // 渲染日志UI
    pub fn ui(&self, ui: &mut Ui) {
        ui.heading("系统日志");
        ui.separator();
        
        // 日志过滤控件
        ui.horizontal(|ui| {
            // 这里可以添加过滤控件
            if ui.button("清除日志").clicked() {
                if let Some(logger) = self.as_mutex() {
                    if let Ok(mut logger) = logger.lock() {
                        logger.clear();
                    }
                }
            }
        });
        
        ui.separator();
        
        // 日志显示区域
        ScrollArea::vertical().stick_to_bottom(self.auto_scroll).show(ui, |ui| {
            for log in &self.logs {
                // 应用过滤器
                if let Some(level) = self.filter_level {
                    if log.level != level {
                        continue;
                    }
                }
                
                if let Some(ref module) = self.filter_module {
                    if !log.module.contains(module) {
                        continue;
                    }
                }
                
                // 显示日志条目
                ui.horizontal(|ui| {
                    let time_str = log.timestamp.format("%Y-%m-%d %H:%M:%S").to_string();
                    ui.label(RichText::new(time_str).monospace());
                    
                    let level_text = RichText::new(log.level_str())
                        .color(log.level_color())
                        .strong();
                    ui.label(level_text);
                    
                    let module_text = RichText::new(format!("[{}]", log.module));
                    ui.label(module_text);
                    
                    ui.label(&log.message);
                });
            }
        });
    }
    
    // 获取自身的互斥锁引用（用于UI中的按钮回调）
    fn as_mutex(&self) -> Option<Arc<Mutex<Logger>>> {
        None // 在实际使用时会被替换为真实的互斥锁引用
    }
}