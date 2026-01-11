/// Главный модуль лаунчера StoryTime Hub
mod ui;
mod git_ops;
mod mods;

/// Точка входа в приложение
fn main() {
    // Выводим баннер при запуске
    ui::print_banner();

    // Основной цикл программы
    loop {
        let choice = ui::main_menu();
        
        match choice.as_deref() {
            Some("󰆽 Установить моды") => {
                // Установка модов
                if let Some(path) = ui::ask_minecraft_folder() {
                    git_ops::install(&path, false);
                }
            }

            Some("󱂵 Переустановить моды") => {
                // Переустановка модов (удалить все и установить заново)
                if let Some(path) = ui::ask_minecraft_folder() {
                    git_ops::install(&path, true);
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