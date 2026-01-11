use inquire::{Select, Confirm, Text};
use std::path::{Path, PathBuf};
use std::fs;
use dirs;

/// Вывод баннера приложения
pub fn print_banner() {
    println!(r#"
███████╗████████╗ ██████╗ ██████╗ ██╗   ██╗████████╗██╗███╗   ███╗███████╗
██╔════╝╚══██╔══╝██╔═══██╗██╔══██╗╚██╗ ██╔╝╚══██╔══╝██║████╗ ████║██╔════╝
███████╗   ██║   ██║   ██║██████╔╝ ╚████╔╝    ██║   ██║██╔████╔██║█████╗
╚════██║   ██║   ██║   ██║██╔══██╗  ╚██╔╝     ██║   ██║██║╚██╔╝██║██╔══╝
███████╗   ██║   ╚██████╔╝██║  ██║   ██║      ██║   ██║██║ ╚═╝ ██║███████╗
╚══════╝   ╚═╝    ╚═════╝ ╚═╝  ╚═╝   ╚═╝      ╚═╝   ╚═╝╚═╝     ╚═╝╚══════╝
"#);
}

/// Отображение главного меню
pub fn main_menu() -> Option<String> {
    let options = vec![
        "󰆽 Установить моды",
        "󱂵 Переустановить моды",
        "󰅖 Выйти",
    ];

    Select::new("󰍽 Выберите действие:", options)
        .prompt()
        .ok()
        .map(|s| s.to_string())
}

/// Запрос папки Minecraft у пользователя
pub fn ask_minecraft_folder() -> Option<PathBuf> {
    let options = vec![
        "󰝚 Автоматически найти (глубокий поиск)",
        "󰩠 Ввести вручную",
        "󰅖 Отмена",
    ];

    let choice = Select::new("󰍽 Как найти папку Minecraft?", options)
        .prompt()
        .ok()?;

    match choice {
        "󰝚 Автоматически найти (глубокий поиск)" => find_minecraft_ultra_deep(),
        "󰩠 Ввести вручную" => ask_path_manual(),
        _ => None,
    }
}

/// Ультра-глубокий поиск папок Minecraft
fn find_minecraft_ultra_deep() -> Option<PathBuf> {
    let mut minecraft_folders = Vec::new();
    
    // Получаем все возможные корневые директории для поиска
    let search_roots = get_extended_search_roots();
    
    println!("󰆍 Ищу папки Minecraft...");
    
    // Многопоточный поиск в каждой корневой директории
    for root in search_roots {
        if root.exists() {
            find_minecraft_recursive(&root, &mut minecraft_folders, 0, 8);
        }
    }
    
    // Добавляем стандартные пути
    add_standard_paths_extended(&mut minecraft_folders);
    
    if minecraft_folders.is_empty() {
        println!("󰅖 Папки Minecraft не найдены");
        return ask_path_manual();
    }
    
    // Удаляем дубликаты и сортируем
    minecraft_folders.sort_by(|a, b| {
        let a_has = a.to_string_lossy().contains(".minecraft");
        let b_has = b.to_string_lossy().contains(".minecraft");
        b_has.cmp(&a_has).then(a.cmp(b))
    });
    minecraft_folders.dedup();
    
    println!("󰝚 Найдено {} папок Minecraft", minecraft_folders.len());
    
    // Выбор папки пользователем
    select_minecraft_folder(&minecraft_folders)
}

/// Получение расширенного списка корневых директорий для поиска
fn get_extended_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    
    // Домашняя директория пользователя
    if let Some(home) = dirs::home_dir() {
        roots.push(home.clone());
        
        // Для Windows
        #[cfg(target_os = "windows")]
        {
            roots.push(home.join("AppData").join("Roaming"));
            roots.push(home.join("AppData").join("Local"));
            roots.push(home.join("Documents"));
            roots.push(home.join("OneDrive").join("Documents"));
        }
        
        // Для Linux
        #[cfg(target_os = "linux")]
        {
            roots.push(home.join(".local").join("share"));
        }
        
        // Для macOS
        #[cfg(target_os = "macos")]
        {
            roots.push(home.join("Library").join("Application Support"));
        }
    }
    
    // Текущая рабочая директория
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd);
    }
    
    roots
}

/// Рекурсивный поиск папок Minecraft
fn find_minecraft_recursive(dir: &Path, results: &mut Vec<PathBuf>, depth: usize, max_depth: usize) {
    if depth >= max_depth || !dir.exists() {
        return;
    }
    
    // Проверяем, является ли текущая папка .minecraft
    if let Some(dir_name) = dir.file_name() {
        let dir_name_str = dir_name.to_string_lossy().to_lowercase();
        
        if dir_name_str.contains("minecraft") || 
           dir_name_str == ".minecraft" ||
           dir_name_str == "minecraft" ||
           dir_name_str.contains("mc") {
            
            if is_valid_minecraft_folder(dir) {
                if !results.contains(&dir.to_path_buf()) {
                    results.push(dir.to_path_buf());
                }
            }
        }
    }
    
    // Рекурсивно обходим поддиректории
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let skip_dirs = ["node_modules", ".git", ".idea", "target", "build", "__pycache__"];
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy().to_lowercase();
                    if skip_dirs.contains(&name_str.as_str()) || name_str.starts_with('.') {
                        continue;
                    }
                }
                
                find_minecraft_recursive(&path, results, depth + 1, max_depth);
            }
        }
    }
}

/// Проверка, является ли папка валидной папкой Minecraft
fn is_valid_minecraft_folder(path: &Path) -> bool {
    let common_dirs = [
        "saves", "resourcepacks", "shaderpacks", 
        "mods", "versions", "assets", "logs",
        "config", "screenshots"
    ];
    
    let common_files = [
        "options.txt", "launcher_profiles.json"
    ];
    
    // Проверяем наличие хотя бы одной типичной папки
    for dir in &common_dirs {
        if path.join(dir).exists() {
            return true;
        }
    }
    
    // Или наличие типичных файлов
    for file in &common_files {
        if path.join(file).exists() {
            return true;
        }
    }
    
    false
}

/// Добавление расширенного списка стандартных путей
fn add_standard_paths_extended(folders: &mut Vec<PathBuf>) {
    let standard_paths = vec![
        dirs::data_dir().map(|p| p.join(".minecraft")),
        dirs::config_dir().map(|p| p.join(".minecraft")),
        dirs::home_dir().map(|p| p.join(".minecraft")),
        
        #[cfg(target_os = "windows")]
        dirs::home_dir().map(|p| p.join("AppData").join("Roaming").join(".minecraft")),
        
        #[cfg(target_os = "macos")]
        dirs::home_dir().map(|p| p.join("Library").join("Application Support").join("minecraft")),
    ];
    
    for path in standard_paths.into_iter().filter_map(|p| p) {
        if path.exists() && path.is_dir() && !folders.contains(&path) {
            if is_valid_minecraft_folder(&path) {
                folders.push(path);
            }
        }
    }
}

/// Выбор папки Minecraft из найденных
fn select_minecraft_folder(folders: &[PathBuf]) -> Option<PathBuf> {
    let mut options: Vec<String> = folders
        .iter()
        .map(|p| {
            let path_str = p.display().to_string();
            let simplified = simplify_path_display(&path_str);
            
            if path_str.contains("curseforge") {
                format!("󰝚 CurseForge: {}", simplified)
            } else if path_str.contains("multimc") || path_str.contains("prismlauncher") {
                format!("󰝚 MultiMC/Prism: {}", simplified)
            } else if path_str.contains(".minecraft") {
                format!("󰝚 Vanilla: {}", simplified)
            } else {
                format!("󰝚 {}", simplified)
            }
        })
        .collect();
    
    options.push("󰩠 Ввести путь вручную".to_string());
    
    let choice = Select::new("󰍽 Выберите папку Minecraft:", options)
        .prompt()
        .ok()?;
    
    // Если выбрана ручная настройка
    if choice == "󰩠 Ввести путь вручную" {
        return ask_path_manual();
    }
    
    // Находим выбранную папку
    for p in folders.iter() {
        let path_str = p.display().to_string();
        let simplified = simplify_path_display(&path_str);
        
        let option_str = if path_str.contains("curseforge") {
            format!("󰝚 CurseForge: {}", simplified)
        } else if path_str.contains("multimc") || path_str.contains("prismlauncher") {
            format!("󰝚 MultiMC/Prism: {}", simplified)
        } else if path_str.contains(".minecraft") {
            format!("󰝚 Vanilla: {}", simplified)
        } else {
            format!("󰝚 {}", simplified)
        };
        
        if option_str == choice {
            return Some(p.clone());
        }
    }
    
    None
}

/// Упрощение отображения пути для пользователя
fn simplify_path_display(path: &str) -> String {
    let home = dirs::home_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    
    if path.starts_with(&home) {
        format!("~{}", &path[home.len()..])
    } else {
        path.to_string()
    }
}

/// Ручной ввод пути к папке Minecraft
fn ask_path_manual() -> Option<PathBuf> {
    Text::new("󰍽 Введите путь к папке Minecraft:")
        .with_help_message("Пример: C:\\Users\\Имя\\.minecraft или /home/пользователь/.minecraft")
        .with_autocomplete(autocomplete_minecraft_paths())
        .prompt()
        .ok()
        .map(PathBuf::from)
        .and_then(|p| {
            let p_clone = p.clone();
            let path = if p.is_relative() {
                std::env::current_dir()
                    .ok()
                    .map(|cwd| cwd.join(p_clone))
                    .unwrap_or(p)
            } else {
                p
            };
            
            if path.exists() && path.is_dir() {
                if is_valid_minecraft_folder(&path) {
                    Some(path)
                } else {
                    let use_anyway = Confirm::new("󰍽 Эта папка не похожа на Minecraft. Использовать всё равно?")
                        .with_default(false)
                        .prompt()
                        .unwrap_or(false);
                    
                    if use_anyway {
                        Some(path)
                    } else {
                        ask_path_manual()
                    }
                }
            } else {
                let try_again = Confirm::new("󰍽 Папка не существует. Попробовать снова?")
                    .with_default(true)
                    .prompt()
                    .unwrap_or(false);
                
                if try_again {
                    ask_path_manual()
                } else {
                    None
                }
            }
        })
}

/// Автодополнение путей для ручного ввода
fn autocomplete_minecraft_paths() -> impl inquire::Autocomplete {
    #[derive(Clone)]
    struct MinecraftPathCompleter;
    
    impl inquire::Autocomplete for MinecraftPathCompleter {
        fn get_suggestions(&mut self, input: &str) -> Result<Vec<String>, inquire::CustomUserError> {
            let mut suggestions = Vec::new();
            
            let standard_paths = vec![
                "~/.minecraft",
                "~/AppData/Roaming/.minecraft",
                "C:\\Users\\%USERNAME%\\.minecraft",
                "/home/%USER%/.minecraft",
            ];
            
            for path in standard_paths {
                if path.contains(input) || input.is_empty() {
                    suggestions.push(path.replace("%USERNAME%", &whoami::username())
                        .replace("%USER%", &whoami::username()));
                }
            }
            
            Ok(suggestions)
        }
        
        fn get_completion(
            &mut self,
            _input: &str,
            highlighted_suggestion: Option<String>,
        ) -> Result<inquire::autocompletion::Replacement, inquire::CustomUserError> {
            match highlighted_suggestion {
                Some(s) => Ok(inquire::autocompletion::Replacement::Some(s)),
                None => Ok(inquire::autocompletion::Replacement::None),
            }
        }
    }
    
    MinecraftPathCompleter
}

/// Выбор экземпляра/папки для установки модов
pub fn select_instance(minecraft_path: &Path) -> Option<PathBuf> {
    // Возможные пути к папкам с модами
    let possible_mods_paths = vec![
        minecraft_path.join("mods"),                    // Прямая папка mods
        minecraft_path.join("minecraft").join("mods"),  // Внутри minecraft
        minecraft_path.join(".minecraft").join("mods"), // Внутри .minecraft
    ];
    
    let mut available_paths = Vec::new();
    
    // Проверяем каждый возможный путь
    for path in possible_mods_paths {
        if path.exists() {
            available_paths.push((path.clone(), "Существующая папка"));
        }
    }
    
    // Если ничего не найдено, предлагаем создать папку mods
    if available_paths.is_empty() {
        let mods_path = minecraft_path.join("mods");
        let create_mods = Confirm::new("󰍽 Не найдено папки mods. Создать в текущей директории?")
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
        let (path, _) = &available_paths[0];
        return Some(path.clone());
    }
    
    // Предлагаем выбор пользователю
    let options: Vec<String> = available_paths
        .iter()
        .map(|(path, description)| {
            format!("󰝚 {} - {}", path.display(), description)
        })
        .collect();
    
    let choice = Select::new("󰍽 Выберите куда установить моды:", options)
        .prompt()
        .ok()?;
    
    for (path, description) in &available_paths {
        let option_str = format!("󰝚 {} - {}", path.display(), description);
        
        if option_str == choice {
            return Some(path.clone());
        }
    }
    
    None
}

/// Выбор типа сборки (клиент/сервер)
pub fn select_build_type() -> Option<String> {
    let options = vec![
        "󰌌 Клиентская сборка",
        "󰑓 Серверная сборка",
    ];

    Select::new("󰍽 Выберите тип сборки для установки:", options)
        .prompt()
        .ok()
        .map(|s| s.to_string())
}