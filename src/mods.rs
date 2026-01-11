use std::fs;
use std::path::Path;
use std::io;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};

/// Установка модов с отображением прогресса
pub fn install_mods_with_progress(
    repo_dir: &Path, 
    mods_dir: &Path, 
    multi_progress: &MultiProgress,
) -> io::Result<u32> {
    // Получаем список файлов для установки
    let files: Vec<_> = fs::read_dir(repo_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().and_then(|e| e.to_str()) == Some("jar"))
        .collect();
    
    if files.is_empty() {
        return Ok(0);
    }
    
    // Создаем прогресс-бар с анимацией как у Docker
    let pb = multi_progress.add(ProgressBar::new_spinner());
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    );
    pb.set_message("Начинаю установку...");
    pb.enable_steady_tick(Duration::from_millis(80));
    
    let mut installed_count = 0;
    
    // Устанавливаем каждый мод
    for (i, file_path) in files.iter().enumerate() {
        let file_name = file_path.file_name().unwrap().to_string_lossy();
        
        // Пропускаем моды с амперсандом в начале
        if file_name.starts_with('&') {
            pb.set_message(format!("Пропускаю: {}", file_name));
            pb.inc(1);
            continue;
        }
        
        let target = mods_dir.join(file_path.file_name().unwrap());
        
        // Обновляем сообщение каждые 5 файлов
        if i % 5 == 0 {
            pb.set_message(format!("Установлено {}/{}", i, files.len()));
        }
        
        // Копируем файл
        match fs::copy(&file_path, &target) {
            Ok(_) => {
                installed_count += 1;
            }
            Err(e) => {
                pb.println(format!("󰅖 Ошибка при установке {}: {}", file_name, e));
            }
        }
        
        pb.inc(1);
    }
    
    pb.finish_with_message(format!("Установка завершена!"));
    Ok(installed_count)
}

/// Очистка всех .jar файлов в директории
pub fn clean_mods_dir(mods_dir: &Path) -> io::Result<u32> {
    let mut removed_count = 0;
    
    if !mods_dir.exists() {
        return Ok(0);
    }
    
    for entry in fs::read_dir(mods_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "jar" {
                    fs::remove_file(&path)?;
                    removed_count += 1;
                }
            }
        }
    }
    
    Ok(removed_count)
}