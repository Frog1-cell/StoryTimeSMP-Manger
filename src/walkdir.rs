use walkdir::WalkDir;
use std::path::Path;

pub fn find_all_minecraft_folders() -> Vec<std::path::PathBuf> {
    let mut folders = Vec::new();
    
    if let Some(home) = dirs::home_dir() {
        for entry in WalkDir::new(&home)
            .max_depth(3) // Ограничиваем глубину поиска для производительности
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_dir() {
                let dir_name = path.file_name().and_then(|n| n.to_str());
                if dir_name == Some(".minecraft") {
                    folders.push(path.to_path_buf());
                }
            }
        }
    }
    
    folders
}