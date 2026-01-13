#![windows_subsystem = "windows"]

mod bepinex;
mod config;
mod poller;
mod types;

use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use config::Config;

use anyhow::{Context, Result, anyhow};
use eframe::{NativeOptions, Renderer, egui};
use egui::{Align2, Color32, TextureHandle};
use serde::Deserialize;
use walkdir::WalkDir;
use zip::read::ZipArchive;

use types::{AppState, ModEntry, Tab};

impl AppState {
    fn log(&mut self, msg: &str) {
        self.status_log.push(msg.to_string());
    }

    fn load_logo_if_needed(&mut self, ctx: &egui::Context) {
        if self.logo_texture.is_some() {
            return;
        }
        const BYTES: &[u8] =
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/logo.png"));
        if let Some(tex) = load_texture_from_png_bytes(ctx, BYTES) {
            self.logo_texture = Some(Box::new(tex));
        }
    }

    fn install_mod_from_zip_path(&mut self, zip_path: &Path) -> Result<()> {
        bepinex::ensure_dirs(&self.game_dir)?;
        let file =
            File::open(zip_path).with_context(|| format!("Open zip {}", zip_path.display()))?;
        let mut zip = ZipArchive::new(file)?;

        let mut installed_files: Vec<String> = Vec::new();
        let mut mod_name: Option<String> = None;
        let mut mod_version: Option<String> = None;

        for i in 0..zip.len() {
            let mut f = zip.by_index(i)?;
            let name = f.name().to_string();
            if name.to_lowercase().ends_with("manifest.json") {
                let mut s = String::new();
                f.read_to_string(&mut s)?;
                #[derive(Deserialize)]
                struct Manifest {
                    name: Option<String>,
                    version_number: Option<String>,
                    version: Option<String>,
                }
                if let Ok(mani) = serde_json::from_str::<Manifest>(&s) {
                    mod_name = mani.name.or(mod_name);
                    mod_version = mani.version_number.or(mani.version).or(mod_version);
                }
            }
        }

        for i in 0..zip.len() {
            let mut f = zip.by_index(i)?;
            let raw_name = f.name().to_string();
            let dest_rel = map_mod_zip_entry_to_game_rel(&raw_name);
            if let Some(rel) = dest_rel {
                let outpath = self.game_dir.join(&rel);
                if raw_name.ends_with('/') {
                    std::fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(p) = outpath.parent() {
                        std::fs::create_dir_all(p)?;
                    }
                    let mut out = File::create(&outpath)?;
                    std::io::copy(&mut f, &mut out)?;
                    installed_files.push(rel.to_string_lossy().to_string());
                }
            }
        }

        if installed_files.is_empty() {
            let mut second_pass = ZipArchive::new(File::open(zip_path)?)?;
            for i in 0..second_pass.len() {
                let mut f = second_pass.by_index(i)?;
                let name = f.name().to_string();
                if !name.ends_with('/') && name.to_lowercase().ends_with(".dll") {
                    let outpath = bepinex::plugins_dir(&self.game_dir)
                        .join(Path::new(&name).file_name().unwrap());
                    let mut out = File::create(&outpath)?;
                    std::io::copy(&mut f, &mut out)?;
                    installed_files.push(format!(
                        "BepInEx/plugins/{}",
                        outpath.file_name().unwrap().to_string_lossy()
                    ));
                }
            }
        }

        if installed_files.is_empty() {
            return Err(anyhow!("No installable files found in zip"));
        }

        let id = zip_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("mod")
            .to_string();
        let entry = ModEntry {
            id: id.clone(),
            name: mod_name.clone().unwrap_or_else(|| id.clone()),
            version: mod_version.clone(),
            source_zip: Some(zip_path.display().to_string()),
            installed_files,
        };
        self.mods.mods.retain(|m| m.id != entry.id);
        self.mods.mods.push(entry);
        let _ = bepinex::save_index(&self.game_dir, &self.mods);
        self.log("Mod installed.");
        Ok(())
    }

    fn uninstall_mod(&mut self, idx: usize) {
        if idx >= self.mods.mods.len() {
            return;
        }
        let m = self.mods.mods[idx].clone();
        let mut removed_any = false;
        for rel in &m.installed_files {
            let p = self.game_dir.join(rel);
            if p.exists() {
                let _ = std::fs::remove_file(&p);
                removed_any = true;
            }
        }
        for entry in WalkDir::new(self.game_dir.join("BepInEx"))
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
            .collect::<Vec<_>>()
        {
            if entry.file_type().is_dir() {
                let _ = std::fs::remove_dir(entry.path());
            }
        }
        self.mods.mods.remove(idx);
        let _ = bepinex::save_index(&self.game_dir, &self.mods);
        if removed_any {
            self.log(&format!("Uninstalled {}", m.name));
        }
    }
}

fn load_texture_from_png_bytes(ctx: &egui::Context, bytes: &[u8]) -> Option<TextureHandle> {
    let dyn_img = image::load_from_memory(bytes).ok()?;
    let rgba = dyn_img.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &rgba);
    Some(ctx.load_texture("logo.png", color_image, egui::TextureOptions::LINEAR))
}

fn draw_play_button(ui: &mut egui::Ui, tex: &TextureHandle) -> egui::Response {
    let size = tex.size_vec2();
    let max_w = 520.0f32;
    let scale = (max_w / size.x).min(1.0);
    let desired = egui::vec2(size.x * scale, size.y * scale);
    let (rect, response) = ui.allocate_at_least(desired, egui::Sense::click());
    if ui.is_rect_visible(rect) {
        let painter = ui.painter_at(rect);
        painter.image(
            tex.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            Color32::WHITE,
        );
        let label = "Play";
        let font = egui::FontId::proportional((desired.y * 0.22).clamp(18.0, 64.0));
        let galley = ui
            .painter()
            .layout_no_wrap(label.to_owned(), font, Color32::WHITE);
        let pos = egui::pos2(
            rect.center().x - galley.size().x / 2.0,
            rect.center().y - galley.size().y / 2.0,
        );
        painter.galley(pos + egui::vec2(2.0, 2.0), galley.clone(), Color32::BLACK);
        painter.galley(pos, galley, Color32::WHITE);
    }
    response
}

fn draw_drop_zone(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let desired = egui::vec2(ui.available_width(), 120.0);
    let (rect, response) = ui.allocate_exact_size(desired, egui::Sense::hover());
    let bg = if response.hovered() {
        Color32::from_rgb(70, 70, 80)
    } else {
        Color32::from_rgb(50, 50, 60)
    };
    ui.painter().rect_filled(rect, 8.0, bg);
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        egui::FontId::proportional(18.0),
        Color32::WHITE,
    );
    response
}

fn launch_game(app: &mut AppState) {
    let exe = app.game_dir.join("Restaurats.exe");
    if exe.exists() {
        let _ = Command::new(&exe)
            .current_dir(&app.game_dir)
            .spawn()
            .map_err(|e| app.log(&format!("Launch failed: {e}")));
    } else {
        app.log("Restaurats.exe not found in game directory");
    }
}

fn open_directory_in_explorer(path: &Path) {
    let _ = open::that(path);
}

fn ui_getting_started(app: &mut AppState, ui: &mut egui::Ui) {
    ui.heading("BepInEx");
    ui.horizontal(|ui| {
        if app.bep_status.is_empty() {
            app.bep_status = bepinex::detect_bep_status(&app.game_dir);
        }
        ui.label(format!("Status: {}", app.bep_status));
        let auto_btn = ui.add_enabled(
            !app.is_busy,
            egui::Button::new("Install Bleeding Edge (auto)"),
        );
        if auto_btn.clicked() {
            app.start_install_bepinex_stable_v5_async();
        }
    });
    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut app.custom_bep_url)
            .on_hover_text("Custom BepInEx zip URL (e.g., BE IL2CPP build)");
        let from_url_btn = ui.add_enabled(!app.is_busy, egui::Button::new("Install from URL"));
        if from_url_btn.clicked() {
            if !app.custom_bep_url.trim().is_empty() {
                let url = app.custom_bep_url.trim().to_string();
                app.start_install_bepinex_from_url_async(url);
            }
        }
        let from_zip_btn = ui.add_enabled(!app.is_busy, egui::Button::new("Install from ZIP..."));
        if from_zip_btn.clicked() {
            if let Some(zip) = rfd::FileDialog::new()
                .add_filter("zip", &["zip"])
                .pick_file()
            {
                app.is_busy = true;
                let game_dir = app.game_dir.clone();
                let task: Arc<Mutex<Option<Result<(), String>>>> = Arc::new(Mutex::new(None));
                app.install_task = Some(task.clone());
                std::thread::spawn(move || {
                    let res = (|| -> Result<()> {
                        let mut buf = Vec::new();
                        File::open(&zip)?.read_to_end(&mut buf)?;
                        bepinex::install_bepinex_from_zip_bytes(&game_dir, &buf)?;
                        if let Err(e) = bepinex::validate_bepinex_installation(&game_dir) {
                            return Err(e);
                        }
                        Ok(())
                    })();
                    *task.lock().unwrap() = Some(res.map_err(|e| e.to_string()));
                });
            }
        }
    });
}

fn ui_mods(app: &mut AppState, ui: &mut egui::Ui, _ctx: &egui::Context) {
    let _ = draw_drop_zone(ui, "Drag a mod zip or dll here");
    ui.add_space(8.0);
    ui.heading("Installed Mods");
    if app.mods.mods.is_empty() {
        ui.label("Drag a mod zip or dll into the box above to install.");
    }
    egui::ScrollArea::vertical()
        .id_salt("mods_scroll")
        .max_height(app.config.constants.mods_max_height)
        .show(ui, |ui| {
            egui::Grid::new("mods_grid").striped(true).show(ui, |ui| {
                ui.label("Name");
                ui.label("Version");
                ui.label("");
                ui.end_row();
                for i in 0..app.mods.mods.len() {
                    let m = &app.mods.mods[i];
                    ui.label(&m.name);
                    ui.label(m.version.clone().unwrap_or_default());
                    if ui.button("Uninstall").clicked() {
                        app.uninstall_mod(i);
                    }
                    ui.end_row();
                }
            });
        });
}

fn map_mod_zip_entry_to_game_rel(entry: &str) -> Option<PathBuf> {
    let lower = entry.to_lowercase();
    if lower.ends_with('/') {
        return Some(PathBuf::from(entry));
    }
    if lower.contains("bepinex/plugins/") {
        let rel = entry
            .splitn(2, |c| c == ':' || c == '*')
            .next()
            .unwrap_or(entry);
        return Some(PathBuf::from(rel));
    }
    if lower.starts_with("plugins/") {
        return Some(PathBuf::from("BepInEx").join("plugins").join(&entry[8..]));
    }
    if lower.starts_with("bepinex/") {
        return Some(PathBuf::from(entry));
    }
    None
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Check if background poller detected BepInEx readiness
        if !self.bep_ready {
            if let Some(flag) = &self.poller_flag {
                if *flag.lock().unwrap() {
                    self.bep_ready = true;
                    self.bep_status = bepinex::detect_bep_status(&self.game_dir);
                    self.log("BepInEx is now ready! Mods tab enabled.");
                    self.poller_flag = None;
                }
            }
        }

        // Check for completion of background install task
        let task_opt = self.install_task.as_ref().map(|a| Arc::clone(a));
        if let Some(task) = task_opt {
            let res_opt = { task.lock().unwrap().take() };
            if let Some(res) = res_opt {
                self.is_busy = false;
                match res {
                    Ok(()) => {
                        self.bep_status = bepinex::detect_bep_status(&self.game_dir);
                        self.bep_ready = bepinex::is_bep_installed(&self.game_dir);
                        self.log("BepInEx installed and validated.");
                        if !self.bep_ready {
                            self.log("Checking for BepInEx readiness in background...");
                            let poller = poller::BepInExPoller::new(self.game_dir.clone());
                            self.poller_flag = Some(poller.start());
                        }
                    }
                    Err(msg) => {
                        self.log(&format!("BepInEx install failed: {}", msg));
                    }
                }
                self.install_task = None;
            }
        }

        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if !dropped.is_empty() && !self.is_busy {
            if self.bep_ready {
                for f in dropped {
                    if let Some(path) = f.path {
                        let is_zip = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .map(|e| e.eq_ignore_ascii_case("zip"))
                            .unwrap_or(false);
                        let is_dll = path
                            .extension()
                            .and_then(|e| e.to_str())
                            .map(|e| e.eq_ignore_ascii_case("dll"))
                            .unwrap_or(false);
                        if is_zip {
                            if let Err(e) = self.install_mod_from_zip_path(&path) {
                                self.log(&format!("Install failed: {e}"));
                            }
                        } else if is_dll {
                            let dest = bepinex::plugins_dir(&self.game_dir)
                                .join(path.file_name().unwrap());
                            if let Some(parent) = dest.parent() {
                                let _ = std::fs::create_dir_all(parent);
                            }
                            match std::fs::copy(&path, &dest) {
                                Ok(_) => {
                                    self.mods.mods.push(ModEntry {
                                        id: path
                                            .file_stem()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("mod")
                                            .to_string(),
                                        name: path
                                            .file_name()
                                            .and_then(|s| s.to_str())
                                            .unwrap_or("mod")
                                            .to_string(),
                                        version: None,
                                        source_zip: Some(path.display().to_string()),
                                        installed_files: vec![format!(
                                            "BepInEx/plugins/{}",
                                            dest.file_name().unwrap().to_string_lossy()
                                        )],
                                    });

                                    let _ = bepinex::save_index(&self.game_dir, &self.mods);
                                    self.log("Mod installed.");
                                }
                                Err(e) => self.log(&format!("Install failed: {e}")),
                            }
                        } else {
                            self.log("Only .zip or .dll files are supported.");
                        }
                    }
                }
            } else {
                self.log("Install BepInEx first to manage mods.");
            }
        }

        egui::TopBottomPanel::top("top").show(ctx, |ui| {
            ui.heading("Restaurats Mod Manager");
        });

        egui::SidePanel::right("play_panel")
            .resizable(false)
            .min_width(200.0)
            .show(ctx, |ui| {
                ui.heading("Play");
                ui.separator();
                self.load_logo_if_needed(ctx);
                if let Some(tex_boxed) = &self.logo_texture {
                    if let Some(tex) = tex_boxed.downcast_ref::<TextureHandle>() {
                        let resp = draw_play_button(ui, tex);
                        if resp.clicked() {
                            launch_game(self);
                        }
                    } else if ui.button("Play Restaurats").clicked() {
                        launch_game(self);
                    }
                } else if ui.button("Play Restaurats").clicked() {
                    launch_game(self);
                }
                ui.separator();
                if ui.button("Browse Game Directory").clicked() {
                    open_directory_in_explorer(&self.game_dir);
                }
                if ui.button("Browse Plugin Directory").clicked() {
                    let plugins_dir = bepinex::plugins_dir(&self.game_dir);
                    open_directory_in_explorer(&plugins_dir);
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Game directory:");
                let mut path_str = self.game_dir.display().to_string();
                if ui.text_edit_singleline(&mut path_str).lost_focus() {
                    self.game_dir = PathBuf::from(path_str);
                }
                if ui.button("Browse...").clicked() {
                    if let Some(dir) = rfd::FileDialog::new()
                        .set_directory(&self.game_dir)
                        .pick_folder()
                    {
                        self.game_dir = dir;
                        self.mods = bepinex::load_index(&self.game_dir);
                        self.bep_status = bepinex::detect_bep_status(&self.game_dir);
                        self.bep_ready = bepinex::is_bep_installed(&self.game_dir);
                    }
                }
            });

            ui.separator();
            ui.horizontal(|ui| {
                let getting = ui.selectable_label(
                    matches!(self.current_tab, Tab::GettingStarted),
                    "Getting Started",
                );
                if getting.clicked() {
                    self.current_tab = Tab::GettingStarted;
                }
                let mods_tab = ui.add_enabled(
                    self.bep_ready,
                    egui::Button::new("Mods").selected(matches!(self.current_tab, Tab::Mods)),
                );
                if mods_tab.clicked() && self.bep_ready {
                    self.current_tab = Tab::Mods;
                }
                if !self.bep_ready {
                    mods_tab.on_hover_text("Install BepInEx first");
                }
            });

            ui.separator();
            match self.current_tab {
                Tab::GettingStarted => ui_getting_started(self, ui),
                Tab::Mods => ui_mods(self, ui, ctx),
            }

            ui.separator();
            ui.heading("Log");
            egui::ScrollArea::vertical()
                .id_salt("log_scroll")
                .max_height(self.config.constants.log_max_height)
                .show(ui, |ui| {
                    for line in &self.status_log {
                        ui.label(line);
                    }
                });
        });
    }
}

impl AppState {
    fn start_install_bepinex_stable_v5_async(&mut self) {
        if self.is_busy {
            return;
        }
        self.is_busy = true;
        let url = self.config.constants.bepinex_url.clone();
        let ua = self.config.constants.user_agent.clone();
        let game_dir = self.game_dir.clone();
        let task: Arc<Mutex<Option<Result<(), String>>>> = Arc::new(Mutex::new(None));
        self.install_task = Some(task.clone());
        std::thread::spawn(move || {
            let res = (|| -> Result<()> {
                let bytes = download_bytes_blocking(&url, &ua)?;
                bepinex::install_bepinex_from_zip_bytes(&game_dir, &bytes)?;
                if let Err(e) = bepinex::validate_bepinex_installation(&game_dir) {
                    return Err(e);
                }
                Ok(())
            })();
            *task.lock().unwrap() = Some(res.map_err(|e| e.to_string()));
        });
    }

    fn start_install_bepinex_from_url_async(&mut self, url: String) {
        if self.is_busy {
            return;
        }
        self.is_busy = true;
        let ua = "restaurats-mod-manager";
        let game_dir = self.game_dir.clone();
        let task: Arc<Mutex<Option<Result<(), String>>>> = Arc::new(Mutex::new(None));
        self.install_task = Some(task.clone());
        std::thread::spawn(move || {
            let res = (|| -> Result<()> {
                let bytes = download_bytes_blocking(&url, ua)?;
                bepinex::install_bepinex_from_zip_bytes(&game_dir, &bytes)?;
                if let Err(e) = bepinex::validate_bepinex_installation(&game_dir) {
                    return Err(e);
                }
                Ok(())
            })();
            *task.lock().unwrap() = Some(res.map_err(|e| e.to_string()));
        });
    }
}

fn download_bytes_blocking(url: &str, user_agent: &str) -> Result<Vec<u8>> {
    let rt = tokio::runtime::Runtime::new()?;
    let bytes = rt.block_on(async {
        let client = reqwest::Client::builder()
            .user_agent(user_agent)
            .build()
            .map_err(|e| anyhow!(e))?;
        let resp = client.get(url).send().await.map_err(|e| anyhow!(e))?;
        let resp = resp.error_for_status().map_err(|e| anyhow!(e))?;
        let b = resp.bytes().await.map_err(|e| anyhow!(e))?;
        Ok::<Vec<u8>, anyhow::Error>(b.to_vec())
    })?;
    Ok(bytes)
}

fn main() -> Result<()> {
    // Load configuration (embedded in binary, optional filesystem override)
    let config = Config::load_or_default(Path::new("Config.toml"));

    let mut app = AppState::default();
    app.config = config;
    app.game_dir = PathBuf::from(&app.config.constants.default_game_dir);
    app.bep_status = bepinex::detect_bep_status(&app.game_dir);
    app.mods = bepinex::load_index(&app.game_dir);
    app.bep_ready = bepinex::is_bep_installed(&app.game_dir);

    let app_title = app.config.constants.app_title.clone();
    let mut native_options = NativeOptions::default();
    native_options.renderer = Renderer::Glow;
    eframe::run_native(
        &app_title,
        native_options,
        Box::new(|_cc| Ok(Box::new(app))),
    )
    .map_err(|e| anyhow!("{e}"))
}
