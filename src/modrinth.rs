use reqwest::blocking::Client;
use serde_json::Value;
use std::fs::{self, File};
use std::io::{Write, Read};
use std::path::Path;
use std::time::Duration;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::{Select, Text};
use console::Term;

const MODRINTH_API: &str = "https://api.modrinth.com/v2";

/// Основная функция загрузки модов с Modrinth
pub fn download_mods(minecraft_path: &Path) {
    let term = Term::stdout();
    let _ = term.clear_screen();
    
    println!("󰚨 Загрузка модов с Modrinth");
    println!("=============================\n");
    
    // Запрашиваем поисковый запрос
    let query = match Text::new("󰝚 Введите название мода для поиска:")
        .with_help_message("Например: sodium, iris, fabric-api")
        .prompt()
    {
        Ok(query) if !query.trim().is_empty() => query.trim().to_string(),
        Ok(_) => {
            println!("󰅖 Пустой запрос");
            return;
        }
        Err(_) => {
            println!("󰅖 Поиск отменен");
            return;
        }
    };
    
    // Ищем моды
    println!("󰇚 Ищем моды по запросу '{}'...", query);
    
    let mods = match search_mods(&query) {
        Ok(mods) if !mods.is_empty() => mods,
        Ok(_) => {
            println!("󰅖 Моды не найдены");
            return;
        }
        Err(e) => {
            println!("󰅖 Ошибка поиска: {}", e);
            return;
        }
    };
    
    // Показываем список найденных модов
    let options: Vec<String> = mods.iter()
        .map(|(title, desc, _)| format!("{} - {}", title, desc))
        .collect();
    
    let selection = match Select::new("󰝚 Выберите мод для загрузки:", options)
        .with_page_size(10)
        .prompt()
    {
        Ok(choice) => choice,
        Err(_) => {
            println!("󰅖 Выбор отменен");
            return;
        }
    };
    
    // Находим выбранный мод
    let (_, _, project_id) = mods.iter()
        .find(|(title, desc, _)| format!("{} - {}", title, desc) == selection)
        .unwrap();
    
    // Запрашиваем версию Minecraft
    let version = match Text::new("󰝚 Введите версию Minecraft (например: 1.20.1):")
        .with_default("1.20.1")
        .prompt()
    {
        Ok(v) => v,
        Err(_) => {
            println!("󰅖 Отменено");
            return;
        }
    };
    
    // Запрашиваем лоадер
    let loader_options = vec!["fabric", "forge", "quilt", "neoforge"];
    let loader = match Select::new("󰝚 Выберите лоадер:", loader_options)
        .prompt()
    {
        Ok(loader) => loader,
        Err(_) => {
            println!("󰅖 Отменено");
            return;
        }
    };
    
    // Получаем информацию о версиях мода
    println!("󰇚 Получаю информацию о версиях...");
    let versions = match get_mod_versions(project_id, &version, loader) {
        Ok(versions) if !versions.is_empty() => versions,
        Ok(_) => {
            println!("󰅖 Нет версий для {} с лоадером {}", version, loader);
            return;
        }
        Err(e) => {
            println!("󰅖 Ошибка: {}", e);
            return;
        }
    };
    
    // Выбираем версию для загрузки
    let version_options: Vec<String> = versions.iter()
        .map(|(name, filename, _)| format!("{} ({})", name, filename))
        .collect();
    
    let selected_version = match Select::new("󰝚 Выберите версию для загрузки:", version_options)
        .prompt()
    {
        Ok(choice) => choice,
        Err(_) => {
            println!("󰅖 Выбор отменен");
            return;
        }
    };
    
    // Находим URL для скачивания
    let (_, _, download_url) = versions.iter()
        .find(|(name, filename, _)| format!("{} ({})", name, filename) == selected_version)
        .unwrap();
    
    // Определяем папку для загрузки
    let mods_path = minecraft_path.join("mods");
    if !mods_path.exists() {
        if let Err(e) = fs::create_dir_all(&mods_path) {
            println!("󰅖 Ошибка создания папки mods: {}", e);
            return;
        }
    }
    
    // Скачиваем мод
    println!("󰇚 Скачиваю мод...");
    match download_file(download_url, &mods_path) {
        Ok(filename) => {
            println!("󰄬 Успешно скачан: {}", filename);
            println!("󰝚 Нажмите Enter чтобы продолжить...");
            let _ = std::io::stdin().read_line(&mut String::new());
        }
        Err(e) => {
            println!("󰅖 Ошибка скачивания: {}", e);
            println!("󰝚 Нажмите Enter чтобы продолжить...");
            let _ = std::io::stdin().read_line(&mut String::new());
        }
    }
}

/// Поиск модов на Modrinth
fn search_mods(query: &str) -> Result<Vec<(String, String, String)>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("{}/search?query={}&limit=20", MODRINTH_API, query);
    
    let response = client.get(&url)
        .timeout(Duration::from_secs(10))
        .send()?;
    
    let json: Value = response.json()?;
    
    let mut results = Vec::new();
    if let Value::Array(hits) = &json["hits"] {
        for hit in hits {
            let title = hit["title"].as_str().unwrap_or("Без названия").to_string();
            let description = hit["description"].as_str().unwrap_or("Без описания")
                .chars().take(60).collect::<String>() + "...";
            let project_id = hit["project_id"].as_str().unwrap_or("").to_string();
            
            results.push((title, description, project_id));
        }
    }
    
    Ok(results)
}

/// Получение версий мода для конкретной версии Minecraft
fn get_mod_versions(
    project_id: &str, 
    minecraft_version: &str,
    loader: &str
) -> Result<Vec<(String, String, String)>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("{}/project/{}/version", MODRINTH_API, project_id);
    
    let response = client.get(&url)
        .timeout(Duration::from_secs(10))
        .send()?;
    
    let versions: Value = response.json()?;
    
    let mut compatible_versions = Vec::new();
    if let Value::Array(versions_array) = versions {
        for version in versions_array {
            let game_versions = version["game_versions"].as_array();
            let loaders_array = version["loaders"].as_array();
            
            // Проверяем совместимость с версией Minecraft и выбранным лоадером
            if let (Some(gv), Some(l)) = (game_versions, loaders_array) {
                let has_correct_version = gv.iter().any(|v| v.as_str() == Some(minecraft_version));
                let has_loader = l.iter().any(|l| l.as_str() == Some(loader));
                
                if has_correct_version && has_loader {
                    let name = version["name"].as_str().unwrap_or("Без названия").to_string();
                    
                    // Получаем массив файлов
                    if let Some(files) = version["files"].as_array() {
                        if let Some(file) = files.first() {
                            let filename = file["filename"].as_str().unwrap_or("mod.jar").to_string();
                            let url = file["url"].as_str().unwrap_or("").to_string();
                            
                            compatible_versions.push((name, filename, url));
                        }
                    }
                }
            }
        }
    }
    
    // Сортируем по имени (сначала свежие версии)
    compatible_versions.sort_by(|a, b| b.0.cmp(&a.0));
    
    Ok(compatible_versions)
}

/// Скачивание файла с отображением прогресса
fn download_file(url: &str, destination: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut response = client.get(url)
        .timeout(Duration::from_secs(30))
        .send()?;
    
    let total_size = response.content_length().unwrap_or(0);
    let filename = url.split('/').last().unwrap_or("mod.jar");
    let filepath = destination.join(filename);
    
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));
    
    let mut file = File::create(&filepath)?;
    let mut buffer = [0; 8192]; // 8KB buffer
    let mut downloaded: u64 = 0;
    
    loop {
        let bytes_read = response.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        
        file.write_all(&buffer[..bytes_read])?;
        downloaded += bytes_read as u64;
        pb.set_position(downloaded);
    }
    
    pb.finish_with_message(format!("󰄬 Скачано: {}", filename));
    Ok(filename.to_string())
}