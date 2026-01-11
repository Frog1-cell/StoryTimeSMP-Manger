use git2::Repository;
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio::runtime::Runtime;

use crate::mods;
use crate::ui;

const CLIENT_REPO_URL: &str = "https://github.com/Frog1-cell/StoryTime-ServerKlient-Mods.git";
const SERVER_REPO_URL: &str = "https://github.com/Frog1-cell/StoryTime-ServerBuild-Mods.git";
const TEMP_DIR: &str = ".storytime-mods-temp";

/// Установка модов в выбранную папку Minecraft
pub fn install(minecraft_path: &Path, clean_install: bool) {
    // Выбор типа сборки (клиентская или серверная)
    let repo_url = match ui::select_build_type() {
        Some(choice) => {
            if choice == "󰌌 Клиентская сборка" {
                CLIENT_REPO_URL
            } else {
                SERVER_REPO_URL
            }
        }
        None => return,
    };

    // Выбор экземпляра/папки для установки модов
    let mods_path: std::path::PathBuf = match ui::select_instance(minecraft_path) {
        Some(path) => path,
        None => return,
    };

    // Создание папки mods, если она не существует
    if !mods_path.exists() {
        if let Err(_) = fs::create_dir_all(&mods_path) {
            return;
        }
    }

    // Если clean_install=true, удаляем все .jar файлы из папки mods
    if clean_install {
        let spinner = create_docker_spinner("󰅖 Очищаю папку модов...");
        match mods::clean_mods_dir(&mods_path) {
            Ok(count) => {
                spinner.finish_with_message(format!("󰄬 Удалено {} модов", count));
            }
            Err(e) => {
                spinner.finish_with_message(format!("󰅖 Ошибка: {}", e));
            }
        }
    }

    // Путь для временного хранения репозитория
    let repo_path = minecraft_path.join(TEMP_DIR);

    // Очистка временной папки при повторной установке
    if repo_path.exists() {
        fs::remove_dir_all(&repo_path).ok();
    }

    // Создание многопоточного прогресс-бара
    let multi_progress = MultiProgress::new();
    
    // Инициализация асинхронного рантайма
    let rt = Runtime::new().unwrap();
    
    // Скачивание репозитория с прогрессом
    let spinner1 = create_docker_spinner("󰇚 Подключаюсь к репозиторию...");
    match rt.block_on(download_repo(repo_url, &repo_path)) {
        Ok(_) => spinner1.finish_with_message("󰄬 Репозиторий скачан!"),
        Err(e) => {
            spinner1.finish_with_message(format!("󰅖 Ошибка: {}", e));
            return;
        }
    }

    // Установка модов с прогрессом
    let spinner2 = create_docker_spinner("󰇚 Устанавливаю моды...");
    match mods::install_mods_with_progress(&repo_path, &mods_path, &multi_progress) {
        Ok(count) => {
            spinner2.finish_with_message(format!("󰄬 Установлено {} модов!", count));
        }
        Err(e) => {
            spinner2.finish_with_message(format!("󰅖 Ошибка: {}", e));
        }
    }

    // Очистка временной папки
    fs::remove_dir_all(&repo_path).ok();
}

/// Асинхронное скачивание репозитория
async fn download_repo(repo_url: &str, repo_path: &Path) -> Result<(), String> {
    // Клонирование репозитория
    Repository::clone(repo_url, repo_path)
        .map_err(|e| format!("Ошибка клонирования: {}", e))?;
    
    Ok(())
}

/// Создание спиннера с анимацией как у Docker
fn create_docker_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::with_template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"]),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}