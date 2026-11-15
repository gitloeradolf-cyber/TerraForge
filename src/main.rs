#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Color32, RichText, Vec2, TextureHandle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() -> eframe::Result<()> {
    let icon = eframe::icon_data::from_png_bytes(
        &std::fs::read("avatar.png").unwrap_or_default()
    ).ok();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 750.0])
            .with_title("TerraForge Launcher")
            .with_icon(icon.unwrap_or_default()),
        ..Default::default()
    };

    eframe::run_native(
        "TerraForge Launcher",
        options,
        Box::new(|cc| {
            let mut visuals = egui::Visuals::dark();
            let forge_orange = Color32::from_rgb(255, 140, 0);
            
            visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(18, 18, 18);
            visuals.widgets.inactive.bg_fill = Color32::from_rgb(25, 25, 25);
            visuals.widgets.hovered.bg_fill = Color32::from_rgb(40, 40, 40);
            visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.5, forge_orange);
            visuals.widgets.active.bg_fill = forge_orange;
            visuals.widgets.active.fg_stroke = egui::Stroke::new(1.5, Color32::BLACK);
            visuals.selection.bg_fill = forge_orange;
            visuals.selection.stroke = egui::Stroke::new(1.0, Color32::BLACK);
            
            cc.egui_ctx.set_visuals(visuals);
            let avatar_texture = load_avatar(&cc.egui_ctx);
            Box::new(TerraForgeApp::new(avatar_texture))
        }),
    )
}

fn load_avatar(ctx: &egui::Context) -> Option<TextureHandle> {
    let img = image::open("avatar.png").ok()?;
    let size = [img.width() as usize, img.height() as usize];
    let image_buffer = img.to_rgba8();
    let pixels = image_buffer.as_flat_samples();
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
    Some(ctx.load_texture("avatar", color_image, egui::TextureOptions::LINEAR))
}

// ФУНКЦИЯ АВТОПОИСКА MINECRAFT
fn find_minecraft() -> Option<PathBuf> {
    // 1. Стандартный путь
    if let Some(appdata) = std::env::var_os("APPDATA") {
        let path = PathBuf::from(appdata).join(".minecraft");
        if path.exists() {
            return Some(path);
        }
    }
    // 2. Путь от Prism/TLauncher (если пользователь там играет)
    if let Some(home) = std::env::var_os("USERPROFILE") {
        let path = PathBuf::from(home).join("AppData").join("Roaming").join(".minecraft");
        if path.exists() {
            return Some(path);
        }
    }
    None
}

// ФУНКЦИЯ ПРОВЕРКИ ВЕРСИИ 26.3
fn check_version_26_3(mc_path: &Path) -> bool {
    let version_path = mc_path.join("versions").join("26.3");
    version_path.exists()
}

#[derive(Serialize, Deserialize, Default)]
struct LauncherConfig {
    username: String,
    password: String,
    remember_me: bool,
    skin_path: String,
    minecraft_path: String,
}

#[derive(PartialEq)]
enum Tab {
    Home,
    Servers,
    Settings,
}

struct TerraForgeApp {
    current_tab: Tab,
    username: String,
    password: String,
    remember_me: bool,
    skin_path: String,
    server_ip: String,
    status: String,
    selected_version: String,
    minecraft_path: String,
    files_found: bool,
    avatar_texture: Option<TextureHandle>,
}

impl Default for Tab {
    fn default() -> Self { Tab::Home }
}

impl TerraForgeApp {
    fn new(avatar_texture: Option<TextureHandle>) -> Self {
        let mut config = LauncherConfig::default();
        let config_path = "launcher_config.json";
        if let Ok(content) = fs::read_to_string(config_path) {
            if let Ok(parsed) = serde_json::from_str::<LauncherConfig>(&content) {
                config = parsed;
            }
        }

        // АВТОПОИСК
        let (final_path, files_found) = if let Some(path) = find_minecraft() {
            let path_str = path.display().to_string();
            let is_valid = check_version_26_3(&path);
            (path_str, is_valid)
        } else {
            (String::new(), false)
        };

        Self {
            current_tab: Tab::Home,
            username: config.username,
            password: config.password,
            remember_me: config.remember_me,
            skin_path: config.skin_path,
            server_ip: String::new(),
            status: if files_found { "Файлы 26.3 найдены!".to_string() } else { "Игра не найдена. Укажите папку.".to_string() },
            selected_version: "TerraForge 26.3".to_string(),
            minecraft_path: if final_path.is_empty() { config.minecraft_path } else { final_path },
            files_found,
            avatar_texture,
        }
    }
}

impl eframe::App for TerraForgeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let forge_orange = Color32::from_rgb(255, 140, 0);
        let dark_bg = Color32::from_rgb(18, 18, 18);
        let panel_bg = Color32::from_rgb(25, 25, 25);
        let text_color = Color32::from_rgb(220, 220, 220);

        // --- ЛЕВАЯ ПАНЕЛЬ ---
        egui::SidePanel::left("left_panel")
            .exact_width(250.0)
            .frame(egui::Frame::none().fill(panel_bg).inner_margin(15.0))
            .show(ctx, |ui| {
                ui.label(RichText::new("TERRA").size(28.0).color(forge_orange).strong());
                ui.label(RichText::new("FORGE").size(28.0).color(Color32::WHITE).strong());
                ui.add_space(20.0);

                egui::Frame::none()
                    .fill(Color32::from_rgb(35, 35, 35))
                    .rounding(5.0)
                    .inner_margin(10.0)
                    .show(ui, |ui| {
                        ui.set_width(220.0);
                        ui.horizontal(|ui| {
                            if let Some(texture) = &self.avatar_texture {
                                let size = Vec2::new(40.0, 40.0);
                                ui.add(egui::Image::new((texture.id(), size)));
                            } else {
                                let (rect, _) = ui.allocate_exact_size(Vec2::splat(40.0), egui::Sense::hover());
                                ui.painter().rect_filled(rect, 5.0, Color32::from_rgb(60, 60, 60));
                            }

                            ui.vertical(|ui| {
                                ui.label(RichText::new(if self.username.is_empty() { "Гость" } else { &self.username }).color(Color32::WHITE).strong());
                                ui.label(RichText::new("Офлайн режим").size(12.0).color(Color32::GRAY));
                            });
                        });
                        ui.add_space(10.0);

                        ui.label(RichText::new("Никнейм:").size(14.0).color(text_color));
                        ui.add_sized([210.0, 25.0], egui::TextEdit::singleline(&mut self.username));
                        ui.add_space(5.0);
                        ui.label(RichText::new("Пароль:").size(14.0).color(text_color));
                        ui.add_sized([210.0, 25.0], egui::TextEdit::singleline(&mut self.password).password(true));
                        
                        ui.add_space(5.0);
                        ui.checkbox(&mut self.remember_me, RichText::new("Запомнить меня").size(14.0).color(text_color));
                    });

                ui.add_space(20.0);
                ui.label(RichText::new("Версия игры").color(Color32::GRAY).size(14.0));
                ui.add_space(5.0);
                egui::ComboBox::from_id_source("version_select")
                    .selected_text(&self.selected_version)
                    .width(220.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_version, "TerraForge 26.3".to_string(), "TerraForge 26.3");
                    });
            });

        // --- ВЕРХНЯЯ ПАНЕЛЬ ---
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::none().fill(dark_bg).inner_margin(egui::Margin::symmetric(15.0, 10.0)))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let nav_btn = |ui: &mut egui::Ui, text: &str, current: bool| {
                        let btn = egui::Button::new(RichText::new(text).color(if current { Color32::BLACK } else { text_color }))
                            .fill(if current { forge_orange } else { Color32::TRANSPARENT });
                        ui.add_sized([100.0, 30.0], btn).clicked()
                    };
                    if nav_btn(ui, "Главная", self.current_tab == Tab::Home) { self.current_tab = Tab::Home; }
                    ui.add_space(10.0);
                    if nav_btn(ui, "Сервера", self.current_tab == Tab::Servers) { self.current_tab = Tab::Servers; }
                    ui.add_space(10.0);
                    if nav_btn(ui, "Настройки", self.current_tab == Tab::Settings) { self.current_tab = Tab::Settings; }
                });
            });

        // --- ЦЕНТРАЛЬНАЯ ПАНЕЛЬ ---
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(dark_bg).inner_margin(20.0))
            .show(ctx, |ui| {
                match self.current_tab {
                    Tab::Home => {
                        egui::Frame::none()
                            .fill(Color32::from_rgb(30, 30, 30))
                            .rounding(8.0)
                            .inner_margin(20.0)
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.label(RichText::new("Добро пожаловать в TerraForge").size(32.0).color(forge_orange).strong());
                                ui.add_space(10.0);
                                ui.label(RichText::new("Клиент Minecraft на Rust с поддержкой высокой производительности.").size(16.0).color(text_color));
                                ui.add_space(20.0);
                                
                                // БЕЗ ГАЛОЧКИ. Только вывод пути или ошибка.
                                if self.files_found {
                                    ui.label(RichText::new(format!("✅ Найдена версия 26.3: {}", self.minecraft_path)).size(14.0).color(Color32::GREEN));
                                } else {
                                    ui.label(RichText::new("❌ Не удалось найти Minecraft 26.3 автоматически.").size(14.0).color(Color32::RED));
                                    ui.add_space(10.0);
                                    ui.horizontal(|ui| {
                                        ui.label("Папка Minecraft:");
                                        ui.add_sized([250.0, 25.0], egui::TextEdit::singleline(&mut self.minecraft_path));
                                        if ui.button("Обзор...").clicked() {
                                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                                self.minecraft_path = path.display().to_string();
                                                self.files_found = check_version_26_3(Path::new(&self.minecraft_path));
                                            }
                                        }
                                    });
                                }

                                ui.add_space(20.0);
                                let play_btn = egui::Button::new(RichText::new("ИГРАТЬ").size(24.0).color(Color32::BLACK).strong())
                                    .fill(forge_orange);
                                if ui.add_sized([200.0, 50.0], play_btn).clicked() {
                                    self.launch_game();
                                }
                            });

                        ui.add_space(20.0);
                        if !self.status.is_empty() {
                            ui.label(RichText::new(&self.status).color(Color32::YELLOW).size(16.0));
                        }
                    }
                    Tab::Servers => {
                        ui.label(RichText::new("Сетевая игра").size(36.0).color(forge_orange).strong());
                        ui.add_space(20.0);
                        egui::Frame::none().fill(panel_bg).rounding(8.0).inner_margin(20.0).show(ui, |ui| {
                            ui.set_width(600.0);
                            ui.label(RichText::new("Прямое подключение").size(20.0).color(Color32::WHITE));
                            ui.add_space(10.0);
                            ui.label("Введите IP сервера:");
                            ui.add_sized([400.0, 35.0], egui::TextEdit::singleline(&mut self.server_ip));
                            ui.add_space(20.0);
                            let connect_btn = egui::Button::new(RichText::new("ПОДКЛЮЧИТЬСЯ").size(18.0).color(Color32::BLACK).strong()).fill(forge_orange);
                            if ui.add_sized([200.0, 45.0], connect_btn).clicked() {
                                self.launch_game();
                            }
                        });
                    }
                    Tab::Settings => {
                        ui.label(RichText::new("Настройки лаунчера").size(36.0).color(forge_orange).strong());
                        ui.add_space(20.0);
                        egui::Frame::none().fill(panel_bg).rounding(8.0).inner_margin(20.0).show(ui, |ui| {
                            ui.set_width(600.0);
                            
                            ui.label(RichText::new("Скин игрока").size(20.0).color(Color32::WHITE));
                            ui.add_space(10.0);
                            
                            ui.horizontal(|ui| {
                                if ui.button(RichText::new("Выбрать файл...").size(16.0)).clicked() {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("Изображения", &["png", "jpg", "jpeg"])
                                        .pick_file() 
                                    {
                                        self.skin_path = path.display().to_string();
                                    }
                                }
                                ui.label(RichText::new(if self.skin_path.is_empty() { "Скин не выбран" } else { &self.skin_path }).color(Color32::GRAY));
                            });

                            ui.add_space(20.0);
                            ui.separator();
                            ui.add_space(20.0);

                            if ui.add(
                                egui::Button::new(
                                    RichText::new("Сохранить настройки")
                                        .size(18.0)
                                        .color(Color32::BLACK)
                                        .strong()
                                ).fill(forge_orange)
                            ).clicked() {
                                self.save_config();
                                self.status = "Настройки сохранены!".to_string();
                            }
                        });
                        
                        if !self.status.is_empty() {
                            ui.add_space(10.0);
                            ui.label(RichText::new(&self.status).color(forge_orange).size(16.0));
                        }
                    }
                }
            });
    }
}

impl TerraForgeApp {
    fn save_config(&self) {
        let config = LauncherConfig {
            username: self.username.clone(),
            password: self.password.clone(),
            remember_me: self.remember_me,
            skin_path: self.skin_path.clone(),
            minecraft_path: self.minecraft_path.clone(),
        };
        if let Ok(json) = serde_json::to_string_pretty(&config) {
            let _ = fs::write("launcher_config.json", json);
        }
    }

    fn launch_game(&mut self) {
        if self.username.is_empty() {
            self.status = "Ошибка: введите никнейм!".to_string();
            return;
        }

        if !self.files_found {
            self.status = "Ошибка: Minecraft 26.3 не найден!".to_string();
            return;
        }

        if self.remember_me {
            self.save_config();
        }

        let game_exe = r"C:\Users\ALT-LX2\Documents\terraforge\horizon-src\target\release\horizon-client.exe";
        let game_dir = format!("{}\\saves", self.minecraft_path);
        let resources_dir = r"C:\Users\ALT-LX2\AppData\Roaming\horizon-launcher\versions\nightly-2026-11-07-2ebf00a\.local\horizon-client\26.3";

        let mut cmd = Command::new(game_exe);
        cmd.arg("--name").arg(&self.username);
        cmd.arg("--game-dir").arg(game_dir);
        cmd.arg("--resources").arg(resources_dir);

        if self.current_tab == Tab::Servers && !self.server_ip.is_empty() {
            cmd.arg("--server").arg(&self.server_ip);
        }

        self.status = format!("Запуск {}...", self.username);
        match cmd.spawn() {
            Ok(_) => self.status = "Игра запущена!".to_string(),
            Err(e) => {
                self.status = "Ошибка: проверь пути к файлам!".to_string();
                println!("Ошибка запуска: {}", e);
            }
        }
    }
}