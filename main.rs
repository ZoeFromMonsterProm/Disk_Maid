use iced::{executor, Alignment, Application, Element, Length, Settings, Theme, Command};
use iced::widget::{button, column, container, pick_list, text, text_input, vertical_space, scrollable};
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub scan_filter: String,
    pub unit: Unit,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            scan_filter: "*".to_string(),
            unit: Unit::MB,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Unit {
    KB,
    MB,
    GB,
}

impl Unit {
    fn convert(&self, bytes: u64) -> f64 {
        match self {
            Unit::KB => bytes as f64 / 1024.0,
            Unit::MB => bytes as f64 / (1024.0 * 1024.0),
            Unit::GB => bytes as f64 / (1024.0 * 1024.0 * 1024.0),
        }
    }
}

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Unit::KB => "KB",
                Unit::MB => "MB",
                Unit::GB => "GB",
            }
        )
    }
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub path: String,
    pub size: u64,
    pub is_dir: bool,
}

fn get_config_path() -> Result<PathBuf, anyhow::Error> {
    let config_dir = dirs::config_dir().ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?;
    let app_dir = config_dir.join("disk-maid-rs"); 
    if !app_dir.exists() {
        fs::create_dir_all(&app_dir)?;
    }
    Ok(app_dir.join("settings.json"))
}

fn load_config() -> Result<AppConfig, anyhow::Error> {
    let path = get_config_path()?;
    if path.exists() {
        let content = fs::read_to_string(path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        Ok(config)
    } else {
        Ok(AppConfig::default())
    }
}

fn save_config(config: &AppConfig) -> Result<(), anyhow::Error> {
    let path = get_config_path()?;
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}

fn scan_directory(path: PathBuf, filter: String) -> Result<Vec<FileInfo>, String> {
    let mut files = Vec::new();
    
    fn scan_recursive(dir: &PathBuf, filter: &str, files: &mut Vec<FileInfo>, depth: usize, max_depth: usize) -> Result<(), String> {
        if depth > max_depth {
            return Ok(());
        }
        if files.len() > 10000 {
            return Ok(());
        }
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return Ok(()),
        };
        
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                let metadata = match fs::metadata(&path) {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let path_str = path.to_string_lossy().to_string();
                
                if metadata.is_dir() {
                    files.push(FileInfo {
                        path: path_str.clone(),
                        size: 0,
                        is_dir: true,
                    });
                    let _ = scan_recursive(&path, filter, files, depth + 1, max_depth);
                } else {
                    let matches = if filter == "*" || filter == "*.*" {
                        true
                    } else if filter.starts_with("*.") {
                        let ext = filter.trim_start_matches("*.");
                        path.extension()
                            .and_then(|e| e.to_str())
                            .map(|e| e.eq_ignore_ascii_case(ext))
                            .unwrap_or(false)
                    } else {
                        true
                    };
                    
                    if matches {
                        files.push(FileInfo {
                            path: path_str,
                            size: metadata.len(),
                            is_dir: false,
                        });
                    }
                }
            }
        }
        Ok(())
    }
    
    scan_recursive(&path, &filter, &mut files, 0, 5)?;
    Ok(files)
}

pub fn main() -> iced::Result {
    DiskViz::run(Settings::default())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    FileScan,
    Settings,
    Help,
}

#[derive(Debug)]
pub struct DiskViz {
    current_screen: Screen,
    config: AppConfig,
    status_message: String,
    scan_filter_buffer: String,
    selected_unit: Unit,
    is_scanning: bool,
    scanned_files: Vec<FileInfo>,
    scan_path_buffer: String,
}

#[derive(Debug, Clone)]
pub enum Message {
    ScreenChanged(Screen),
    StartScanPressed,
    StopScanPressed,
    BackToMainMenu,
    ExitApp,
    ScanPathChanged(String),
    ScanCompleted(Result<Vec<FileInfo>, String>),
    ScanFilterChanged(String),
    UnitChanged(Unit),
    SaveSettingsPressed,
    ConfigSaved(Result<(), String>),
}

impl Application for DiskViz {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let config = load_config().unwrap_or_default();
        let home_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .to_string_lossy()
            .to_string();
        
        (
            DiskViz {
                current_screen: Screen::MainMenu,
                config: config.clone(),
                scan_filter_buffer: config.scan_filter.clone(),
                selected_unit: config.unit,
                is_scanning: false,
                status_message: "Welcome to Disk Maid!".into(),
                scanned_files: Vec::new(),
                scan_path_buffer: home_dir,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Disk Maid".to_string()
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::ScreenChanged(screen) => {
                self.current_screen = screen;
                Command::none()
            }

            Message::BackToMainMenu => {
                self.current_screen = Screen::MainMenu;
                self.status_message = "Welcome to Disk Maid!".into();
                Command::none()
            }

            Message::ExitApp => {
                std::process::exit(0);
            }

            Message::StartScanPressed => {
                let path = PathBuf::from(self.scan_path_buffer.clone());
                if !path.exists() {
                    self.status_message = "Error: Path does not exist!".into();
                    return Command::none();
                }
                if !path.is_dir() {
                    self.status_message = "Error: Path is not a directory!".into();
                    return Command::none();
                }
                
                self.is_scanning = true;
                self.status_message = "Scanning... (limited to 10,000 files and 5 levels deep)".into();
                self.scanned_files.clear();
                
                let filter = self.config.scan_filter.clone();
                
                Command::perform(
                    async move {
                        scan_directory(path, filter)
                    },
                    Message::ScanCompleted
                )
            }

            Message::StopScanPressed => {
                self.is_scanning = false;
                self.status_message = "Scan stopped.".into();
                Command::none()
            }

            Message::ScanPathChanged(path) => {
                self.scan_path_buffer = path;
                Command::none()
            }

            Message::ScanCompleted(Ok(files)) => {
                self.is_scanning = false;
                let file_count = files.iter().filter(|f| !f.is_dir).count();
                let dir_count = files.iter().filter(|f| f.is_dir).count();
                let total_size: u64 = files.iter().filter(|f| !f.is_dir).map(|f| f.size).sum();
                
                self.scanned_files = files;
                self.status_message = format!(
                    "Scan complete! Found {} files and {} directories. Total size: {:.2} {}",
                    file_count,
                    dir_count,
                    self.config.unit.convert(total_size),
                    self.config.unit
                );
                Command::none()
            }

            Message::ScanCompleted(Err(e)) => {
                self.is_scanning = false;
                self.status_message = format!("Scan error: {}", e);
                Command::none()
            }

            Message::ScanFilterChanged(new_filter) => {
                self.scan_filter_buffer = new_filter;
                Command::none()
            }

            Message::UnitChanged(unit) => {
                self.selected_unit = unit;
                Command::none()
            }

            Message::SaveSettingsPressed => {
                self.config.scan_filter = self.scan_filter_buffer.clone();
                self.config.unit = self.selected_unit;
                let config_to_save = self.config.clone();

                self.status_message = "Saving settings...".into();

                Command::perform(
                    async move { save_config(&config_to_save).map_err(|e| e.to_string()) },
                    Message::ConfigSaved
                )
            }

            Message::ConfigSaved(Ok(())) => {
                self.status_message = "Settings saved successfully!".into();
                Command::none()
            }

            Message::ConfigSaved(Err(e)) => {
                self.status_message = format!("Error saving settings: {}", e);
                Command::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let content = match self.current_screen {
            Screen::MainMenu => main_menu_view(),
            Screen::FileScan => file_scan_view(
                self.is_scanning,
                &self.scan_path_buffer,
                &self.scanned_files,
                self.config.unit,
            ),
            Screen::Settings => settings_view(
                &self.scan_filter_buffer,
                self.selected_unit,
            ),
            Screen::Help => help_view(),
        };

        let layout = if self.current_screen != Screen::MainMenu {
            column![
                button(text("Back to Main Menu")).on_press(Message::BackToMainMenu),
                vertical_space(),
                content,
                vertical_space(),
                container(text(&self.status_message))
                    .padding(10)
                    .width(Length::Fill)
            ]
            .padding(20)
            .align_items(Alignment::Start)
        } else {
            column![content]
                .padding(20)
                .align_items(Alignment::Center)
        };

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

fn main_menu_view() -> Element<'static, Message> {
    let file_scan_btn = button(text("File & Scan").size(20))
        .on_press(Message::ScreenChanged(Screen::FileScan))
        .padding(20)
        .width(Length::Fixed(300.0));
    
    let settings_btn = button(text("Settings").size(20))
        .on_press(Message::ScreenChanged(Screen::Settings))
        .padding(20)
        .width(Length::Fixed(300.0));
    
    let help_btn = button(text("Help").size(20))
        .on_press(Message::ScreenChanged(Screen::Help))
        .padding(20)
        .width(Length::Fixed(300.0));
    
    let exit_btn = button(text("Exit").size(20))
        .on_press(Message::ExitApp)
        .padding(20)
        .width(Length::Fixed(300.0))
        .style(iced::theme::Button::Destructive);

    column![
        text("Disk Maid").size(36),
        file_scan_btn,
        settings_btn,
        help_btn,
        exit_btn,
    ]
    .spacing(20)
    .align_items(Alignment::Center)
    .into()
}

fn file_scan_view<'a>(
    is_scanning: bool,
    scan_path: &'a str,
    files: &'a [FileInfo],
    unit: Unit,
) -> Element<'a, Message> {
    let mut col = column![
        text("File & Scan").size(28),
        text("Enter directory path to scan:"),
        text_input("Enter path (e.g., /home/user or C:\\Users)", scan_path)
            .on_input(Message::ScanPathChanged),
    ];

    if !is_scanning {
        col = col.push(button(text("▶ Start Scan")).on_press(Message::StartScanPressed).padding(10));
    } else {
        col = col.push(button(text("⏹ Stop Scan")).on_press(Message::StopScanPressed).padding(10));
        col = col.push(text("Scanning in progress..."));
    }

    if !files.is_empty() {
        col = col.push(text(format!("Found {} items:", files.len())).size(18));

        let mut file_list = column![].spacing(5);
        
        for file in files.iter().take(100) {
            let display_text = if file.is_dir {
                format!("{}", file.path)
            } else {
                format!(
                    " {} ({:.2} {})",
                    file.path,
                    unit.convert(file.size),
                    unit
                )
            };
            file_list = file_list.push(text(display_text).size(12));
        }

        if files.len() > 100 {
            file_list = file_list.push(text(format!("... and {} more items", files.len() - 100)));
        }

        col = col.push(scrollable(file_list).height(Length::Fixed(400.0)));
    }

    col.spacing(15).into()
}

// Signature changed: lang removed
fn settings_view<'a>(filter: &'a str, unit: Unit) -> Element<'a, Message> {
    column![
        text("Settings").size(28),
        text("Scan Filter:"),
        text("(e.g., *.txt for text files, *.jpg for images, * for all files)").size(12),
        text_input("e.g., *.txt or *", filter).on_input(Message::ScanFilterChanged),
        text("Display Unit:"),
        pick_list(
            vec![Unit::KB, Unit::MB, Unit::GB],
            Some(unit),
            Message::UnitChanged
        ),
        button(text("Save Settings"))
            .on_press(Message::SaveSettingsPressed)
            .padding(10)
    ]
    .spacing(15)
    .into()
}

fn help_view() -> Element<'static, Message> {
    column![
        text("Help & About").size(28),
        text("How to Use:").size(20),
        text("Click 'File & Scan' from the main menu"),
        text("Enter a directory path you want to scan"),
        text("Click 'Start Scan' to begin scanning"),
        text("View the list of files and their sizes"),
        text("Settings:").size(20),
        text("• Change the scan filter to scan specific file types"),
        text("• Example filters: *.txt, *.jpg, *.pdf, or * for all files"),
        text("• Change display unit (KB, MB, or GB)"),
        text("• Don't forget to click 'Save Settings'!"),
        text("About:").size(20),
        text("Disk Maid v1.0.0"),
        text("A Rust-based disk scanning and visualization tool"),
        text("Created with heart using Rust + Iced"),
    ]
    .spacing(10)
    .into()
}