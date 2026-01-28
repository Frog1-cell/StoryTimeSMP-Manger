use inquire::{Select, Confirm, Text};
use std::path::{Path, PathBuf};
use std::fs;
use dirs;
use walkdir::WalkDir;
use console::Term;

/// Вывод баннера приложения
pub fn print_banner() {
    println!(r#"
███████╗████████╗███╗   ███╗
██╔════╝╚══██╔══╝████╗ ████║
███████╗   ██║   ██╔████╔██║
╚════██║   ██║   ██║╚██╔╝██║
███████║   ██║   ██║ ╚═╝ ██║
╚══════╝   ╚═╝   ╚═╝     ╚═╝

StoryTime Hub - Лаунчер для Minecraft
"#);
}

/// Отображение главного меню
pub fn main_menu() -> Option<String> {
    let term = Term::stdout();
    let _ = term.clear_screen();
    print_banner();
    
    let options = vec![
        "󰆽 Установить моды",
        "󱂵 Переустановить моды",
        "󰚨 Загрузить моды с Modrinth",
        "󰒓 Установить папку по умолчанию",
        "󰅖 Выйти",
    ];

    Select::new("󰝚 Выберите действие:", options)
        .prompt()
        .ok()
        .map(|s| s.to_string())
}

/// Запрос папки Minecraft у пользователя
pub fn ask_minecraft_folder() -> Option<PathBuf> {
    let term = Term::stdout();
    let _ = term.clear_screen();
    print_banner();
    
    let options = vec![
        "󰝚 Автоматический поиск (глубокий поиск)",
        "󰒓 Ввести путь вручную",
        "󰅖 Отмена",
    ];

    let choice = Select::new("󰝚 Как найти папку Minecraft?", options)
        .prompt()
        .ok()?;

    match choice {
        "󰝚 Автоматический поиск (глубокий поиск)" => find_minecraft_linux(),
        "󰒓 Ввести путь вручную" => ask_path_manual(),
        _ => None,
    }
}

/// Запрос папки Minecraft с предложением пути по умолчанию
pub fn ask_minecraft_folder_with_default(default_path: Option<&PathBuf>) -> Option<PathBuf> {
    let term = Term::stdout();
    let _ = term.clear_screen();
    print_banner();
    
    if let Some(default) = default_path {
        let use_default = Confirm::new(&format!(
            "󰝚 Использовать папку по умолчанию?\n{}",
            default.display()
        ))
        .with_default(true)
        .prompt()
        .unwrap_or(false);
        
        if use_default {
            return Some(default.clone());
        }
    }
    
    ask_minecraft_folder()
}

/// Глубокий поиск папок Minecraft на Linux
fn find_minecraft_linux() -> Option<PathBuf> {
    println!("󰇚 Ищу папки Minecraft...");
    
    let mut found_folders = Vec::new();
    
    // Стандартные пути для различных лаунчеров на Linux
    let search_paths = get_linux_search_paths();
    
    for base_path in search_paths {
        if base_path.exists() {
            find_minecraft_in_directory(&base_path, &mut found_folders);
        }
    }
    
    if found_folders.is_empty() {
        println!("󰅖 Папки Minecraft не найдены");
        return ask_path_manual();
    }
    
    // Сортируем по приоритету
    found_folders.sort_by(|a, b| {
        let a_priority = get_path_priority(a);
        let b_priority = get_path_priority(b);
        b_priority.cmp(&a_priority)
    });
    
    // Удаляем дубликаты
    found_folders.dedup();
    
    println!("󰄬 Найдено {} папок Minecraft", found_folders.len());
    
    // Предлагаем выбор пользователю
    select_folder_from_list(&found_folders)
}

/// Получение путей для поиска на Linux
fn get_linux_search_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    
    if let Some(home) = dirs::home_dir() {
        // Официальный лаунчер
        paths.push(home.join(".minecraft"));
        
        // Prism Launcher
        paths.push(home.join(".local/share/PrismLauncher/instances"));
        paths.push(home.join(".var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher/instances"));
        
        // MultiMC
        paths.push(home.join(".local/share/multimc/instances"));
        
        // ATLauncher
        paths.push(home.join(".local/share/atlutut/instances"));
        
        // CurseForge
        paths.push(home.join(".local/share/curseforge/minecraft/Instances"));
        
        // GDLauncher
        paths.push(home.join(".local/share/gdlauncher/instances"));
        
        // TLauncher
        paths.push(home.join(".local/share/tlauncher/instances"));
        paths.push(home.join(".tlauncher"));
        
        // Просто папка .minecraft в разных местах
        paths.push(home.clone());
        paths.push(home.join("Games"));
        paths.push(home.join("minecraft"));
        paths.push(home.join("MC"));
    }
    
    // Текущая рабочая директория
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd);
    }
    
    paths
}

/// Рекурсивный поиск папок Minecraft в директории
fn find_minecraft_in_directory(dir: &Path, results: &mut Vec<PathBuf>) {
    let walker = WalkDir::new(dir)
        .max_depth(5)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok());
    
    for entry in walker {
        let path = entry.path();
        
        if path.is_dir() {
            // Проверяем, является ли это папкой Minecraft
            if is_minecraft_folder(path) {
                // Находим корневую папку Minecraft
                if let Some(root_path) = find_minecraft_root(path) {
                    if !results.contains(&root_path) {
                        results.push(root_path);
                    }
                }
            }
        }
    }
}

/// Поиск корневой папки Minecraft
fn find_minecraft_root(start_path: &Path) -> Option<PathBuf> {
    let mut current = start_path.to_path_buf();
    
    // Поднимаемся вверх, пока не найдем .minecraft или папку с модами
    while let Some(parent) = current.parent() {
        if is_minecraft_folder(&current) {
            return Some(current);
        }
        current = parent.to_path_buf();
    }
    
    None
}

/// Проверка, является ли папка папкой Minecraft
fn is_minecraft_folder(path: &Path) -> bool {
    let folder_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    // Проверка по имени папки
    let is_named_minecraft = folder_name.contains("minecraft") || 
                            folder_name == ".minecraft" ||
                            folder_name.contains("instance") ||
                            folder_name.contains("mc");
    
    // Проверка по содержимому
    let has_minecraft_content = path.join("mods").exists() ||
                               path.join("saves").exists() ||
                               path.join("versions").exists() ||
                               path.join("assets").exists() ||
                               path.join("options.txt").exists() ||
                               path.join("launcher_profiles.json").exists();
    
    is_named_minecraft || has_minecraft_content
}

/// Приоритет для сортировки путей
fn get_path_priority(path: &Path) -> i32 {
    let path_str = path.display().to_string();
    
    if path_str.contains(".minecraft") { 100 }
    else if path_str.contains("PrismLauncher") { 90 }
    else if path_str.contains("multimc") { 80 }
    else if path_str.contains("tlauncher") { 70 }
    else if path_str.contains("curseforge") { 60 }
    else if path_str.contains("instances") { 50 }
    else if path.join("mods").exists() { 40 }
    else { 10 }
}

/// Выбор папки из списка найденных
fn select_folder_from_list(folders: &[PathBuf]) -> Option<PathBuf> {
    let mut options: Vec<String> = folders.iter()
        .enumerate()
        .map(|(i, path)| {
            let display_path = simplify_path_display(path);
            let launcher = detect_launcher_type(path);
            format!("{}. {} [{}]", i + 1, display_path, launcher)
        })
        .collect();
    
    options.push("󰒓 Ввести путь вручную".to_string());
    options.push("󰅖 Отмена".to_string());
    
    let choice = Select::new("󰝚 Выберите папку Minecraft:", options)
        .with_page_size(15)
        .prompt()
        .ok()?;
    
    if choice == "󰒓 Ввести путь вручную" {
        return ask_path_manual();
    }
    
    if choice == "󰅖 Отмена" {
        return None;
    }
    
    // Извлекаем индекс из выбора
    if let Some(index_str) = choice.split('.').next() {
        if let Ok(index) = index_str.parse::<usize>() {
            if index > 0 && index <= folders.len() {
                return Some(folders[index - 1].clone());
            }
        }
    }
    
    None
}

/// Определение типа лаунчера по пути
fn detect_launcher_type(path: &Path) -> &'static str {
    let path_str = path.display().to_string();
    
    if path_str.contains("PrismLauncher") { "Prism" }
    else if path_str.contains("multimc") { "MultiMC" }
    else if path_str.contains("tlauncher") { "TLauncher" }
    else if path_str.contains("curseforge") { "CurseForge" }
    else if path_str.contains("atlutut") { "ATLauncher" }
    else if path_str.contains("gdlauncher") { "GDLauncher" }
    else if path_str.contains(".minecraft") { "Официальный" }
    else { "Неизвестно" }
}

/// Упрощение отображения пути
fn simplify_path_display(path: &Path) -> String {
    let home = dirs::home_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    
    let path_str = path.display().to_string();
    
    if path_str.starts_with(&home) {
        format!("~{}", &path_str[home.len()..])
    } else {
        path_str
    }
}

/// Ручной ввод пути
fn ask_path_manual() -> Option<PathBuf> {
    let term = Term::stdout();
    let _ = term.clear_screen();
    print_banner();
    
    let path = Text::new("󰝚 Введите путь к папке Minecraft:")
        .with_help_message("Пример: /home/user/.minecraft или /home/user/Games/MC")
        .prompt()
        .ok()?;
    
    let path_buf = PathBuf::from(&path);
    
    if path_buf.exists() && path_buf.is_dir() {
        if is_minecraft_folder(&path_buf) {
            return Some(path_buf);
        } else {
            let use_anyway = Confirm::new("󰝚 Эта папка не похожа на папку Minecraft. Использовать её?")
                .with_default(false)
                .prompt()
                .unwrap_or(false);
            
            if use_anyway {
                return Some(path_buf);
            } else {
                return ask_path_manual();
            }
        }
    } else {
        let create = Confirm::new("󰝚 Папка не существует. Создать?")
            .with_default(false)
            .prompt()
            .unwrap_or(false);
        
        if create {
            if let Err(e) = fs::create_dir_all(&path_buf) {
                println!("󰅖 Ошибка создания папки: {}", e);
                return ask_path_manual();
            }
            return Some(path_buf);
        } else {
            return ask_path_manual();
        }
    }
}

/// Выбор экземпляра/папки для установки модов
pub fn select_instance(minecraft_path: &Path) -> Option<PathBuf> {
    let term = Term::stdout();
    let _ = term.clear_screen();
    print_banner();
    
    // Возможные пути к папкам с модами
    let possible_mods_paths = vec![
        minecraft_path.join("mods"),
        minecraft_path.join("minecraft").join("mods"),
        minecraft_path.join(".minecraft").join("mods"),
    ];
    
    let mut available_paths = Vec::new();
    
    // Проверяем каждый возможный путь
    for path in possible_mods_paths {
        if path.exists() {
            available_paths.push(path.clone());
        }
    }
    
    // Если ничего не найдено, предлагаем создать папку mods
    if available_paths.is_empty() {
        let mods_path = minecraft_path.join("mods");
        let create_mods = Confirm::new("󰝚 Папка mods не найдена. Создать в текущей директории?")
            .with_default(true)
            .prompt()
            .unwrap_or(false);
        
        if create_mods {
            if let Err(_) = fs::create_dir_all(&mods_path) {
                return None;
            }
            return Some(mods_path);
        }
        return None;
    }
    
    // Если только один вариант, выбираем его
    if available_paths.len() == 1 {
        return Some(available_paths[0].clone());
    }
    
    // Предлагаем выбор пользователю
    let options: Vec<String> = available_paths.iter()
        .map(|path| format!("󰝚 {}", path.display()))
        .collect();
    
    let choice = Select::new("󰝚 Выберите папку для установки модов:", options)
        .prompt()
        .ok()?;
    
    for path in &available_paths {
        if format!("󰝚 {}", path.display()) == choice {
            return Some(path.clone());
        }
    }
    
    None
}

/// Выбор типа сборки (клиентская или серверная)
pub fn select_build_type() -> Option<String> {
    let term = Term::stdout();
    let _ = term.clear_screen();
    print_banner();
    
    let options = vec![
        "󰌌 Клиентская сборка",
        "󰑓 Серверная сборка",
    ];

    Select::new("󰝚 Выберите тип сборки для установки:", options)
        .prompt()
        .ok()
        .map(|s| s.to_string())
}