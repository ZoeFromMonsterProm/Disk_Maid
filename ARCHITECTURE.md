1. Overview
Disk Maid is a GUI application written in Rust using the iced GUI library. It is designed as a simple tool for scanning directory structures, filtering files, and displaying the results.
It follows The Elm Architecture (MVU), which is the core design pattern used by iced:
Model: The DiskViz struct holds the entire application state.
View: The DiskViz::view method (and its helpers like file_scan_view) renders the state into a tree of widgets.
Update: The DiskViz::update method processes Message enums to update the state in response to user interaction or asynchronous events.

2. Technology Stack
Core Language: Rust
GUI Framework: iced
Uses iced::Application trait.
Widgets used: button, column, container, pick_list, text, text_input, vertical_space, scrollable.
Asynchronous Runtime: iced::executor::Default (based on tokio).
Serialization: serde (with serde_json) for saving and loading AppConfig.
System Directories: dirs for finding the appropriate user-specific configuration directory.
File System: std::fs and std::path::PathBuf for all file system I/O.

3. Core Components
DiskViz (The Model)
This is the main state struct for the application.
Rust
struct DiskViz {
    current_screen: Screen,     // Tracks the visible screen (Menu, Scan, Settings, Help)
    config: AppConfig,          // The loaded settings (filter, unit)
    status_message: String,     // Text shown at the bottom of the screen
    scan_filter_buffer: String, // Temporary state for the settings text_input
    selected_unit: Unit,        // Temporary state for the settings pick_list
    is_scanning: bool,          // Flag to control UI during scanning
    scanned_files: Vec<FileInfo>, // The results from the last scan
    scan_path_buffer: String,   // The path from the text_input on the scan screen
}
Message (The Update)
This enum defines every possible event that can change the application state.
ScreenChanged(Screen): For navigation.
StartScanPressed, StopScanPressed: To control the scan.
ScanPathChanged(String): When the user types in the path text_input.
ScanCompleted(Result<Vec<FileInfo>, String>): The result from the async scan operation.
ScanFilterChanged(String), UnitChanged(Unit): When the user changes settings.
SaveSettingsPressed: To trigger saving the config.
ConfigSaved(Result<(), String>): The result from the async save operation.
DiskViz::update (The Logic)
This is the core state machine. It matches on an incoming Message and mutates self (the DiskViz state).
A key part of this is handling asynchronous tasks (like scanning and saving) using iced::Command.
Example: Starting a Scan
User clicks "Start Scan", sending Message::StartScanPressed.
The update function:
Validates self.scan_path_buffer (checks if it exists and is a directory).
Sets self.is_scanning = true and updates self.status_message.
Returns a Command::perform(scan_directory(...), Message::ScanCompleted).
iced executes the scan_directory function on a separate thread.
When scan_directory finishes, it returns a Result.
iced wraps this Result in Message::ScanCompleted and sends it back to the update function.
The update function matches on Message::ScanCompleted(Ok(files)):
Sets self.is_scanning = false.
Populates self.scanned_files with the results.
Updates self.status_message with a summary.

4. Directory Scanning (scan_directory)
The scan_directory function is the main "business logic."
It spawns a recursive helper function, scan_recursive.
Limitations: To prevent poor performance or stack overflows on huge directories, the scan is intentionally limited to:
A maximum depth of 5 levels (max_depth: 5).
A maximum file/directory count of 10,000 (files.len() > 10000).
Filtering: The filter logic is basic string matching:
* or *.*: Matches all files.
*.ext: Matches files with the specified extension (case-insensitive).
5. Configuration Management
Path: get_config_path uses the dirs crate to find the OS-specific config directory (e.g., ~/.config/disk-maid-rs/ on Linux).
Struct: AppConfig is the serializable struct holding the settings.
Loading: load_config is called in DiskViz::new to read settings.json and deserialize it using serde_json. If the file doesn't exist, it uses AppConfig::default().
Saving: save_config is called via a Command when Message::SaveSettingsPressed is received. It serializes the AppConfig state to settings.json as pretty-printed JSON.
