/// Главный модуль лаунчера StoryTime Hub
mod ui;
mod git_ops;
mod mods;
mod modrinth;
mod config;

use console::Term;

/// Точка входа в приложение
fn main() {
    let term = Term::stdout();
    
    // Выводим баннер при запуске
    ui::print_banner();
    
    // Загружаем конфигурацию
    let mut config = config::Config::load();
    
    // Основной цикл программы
    loop {
        let _ = term.clear_screen();
        ui::print_banner();
        
        let choice = ui::main_menu();
        
        match choice.as_deref() {
            Some("󰆽 Установить моды") => {
                let _ = term.clear_screen();
                ui::print_banner();
                
                // Установка модов
                let path = match config.get_default_path() {
                    Some(default_path) => ui::ask_minecraft_folder_with_default(Some(&default_path)),
                    None => ui::ask_minecraft_folder(),
                };
                
                if let Some(path) = path {
                    git_ops::install(&path, false);
                }
            }

            Some("󱂵 Переустановить моды") => {
                let _ = term.clear_screen();
                ui::print_banner();
                
                // Переустановка модов (удалить все и установить заново)
                let path = match config.get_default_path() {
                    Some(default_path) => ui::ask_minecraft_folder_with_default(Some(&default_path)),
                    None => ui::ask_minecraft_folder(),
                };
                
                if let Some(path) = path {
                    git_ops::install(&path, true);
                }
            }

            Some("󰚨 Загрузить моды с Modrinth") => {
                let _ = term.clear_screen();
                ui::print_banner();
                
                // Загрузка модов с Modrinth
                let path = match config.get_default_path() {
                    Some(default_path) => ui::ask_minecraft_folder_with_default(Some(&default_path)),
                    None => ui::ask_minecraft_folder(),
                };
                
                if let Some(path) = path {
                    modrinth::download_mods(&path);
                }
            }

            Some("󰒓 Установить папку по умолчанию") => {
                let _ = term.clear_screen();
                ui::print_banner();
                
                // Установка папки по умолчанию
                if let Some(path) = ui::ask_minecraft_folder() {
                    config.set_default_path(&path);
                    println!("󰄬 Папка по умолчанию установлена: {}", path.display());
                    println!("󰝚 Нажмите Enter чтобы продолжить...");
                    let _ = std::io::stdin().read_line(&mut String::new());
                }
            }

            Some("󰅖 Выйти") => {
                // Завершение работы программы
                break;
            }

            _ => {
                // Выход при некорректном выборе
                break;
            }
        }
    }
}