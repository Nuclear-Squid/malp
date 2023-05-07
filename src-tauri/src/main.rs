// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::{ self, File };
use std::io::Write;
use serde::{ Serialize, Deserialize };

#[derive(Serialize, Deserialize)]
struct Config {
    documents_root_repo: String,
}

#[tauri::command]
fn parse_config_file() -> Config {
    let config_file_path = shellexpand::tilde("~/.config/malp_conf.toml");
    let config_contents = fs::read_to_string(config_file_path.as_ref()).unwrap();
    let mut rv: Config = toml::from_str(&config_contents).unwrap();
    rv.documents_root_repo = shellexpand::tilde(&rv.documents_root_repo).as_ref().to_owned();
    rv
}

#[derive(Serialize, Deserialize)]
struct Project {
    /// Name of the project
    name: String,
    /// Path to the dir containing the project (relative to `documents_root_repo`)
    parent_dir_path: String,
}

#[tauri::command]
fn fetch_projects(root_repo: &str, current_dir: &str) -> Vec<Project> {
    // Check if current dir is a project, and return it if it is
    for entry in fs::read_dir(current_dir).unwrap() {
        let Ok(entry) = entry else { continue; };

        if entry.file_name() == "index.md" {
            let (absolute_parent_dir_path, project_name) =
                current_dir.rsplit_once('/').unwrap();
            let parent_dir_path = absolute_parent_dir_path
                    .strip_prefix(root_repo)
                    .unwrap();

            return vec![Project {
                name: project_name.to_owned(),
                parent_dir_path: parent_dir_path.to_owned() + "/",
            }];
        }
    }

    // Current dir isn’t a project, check sub directories
    fs::read_dir(current_dir).unwrap()
        .map(|entry| fetch_projects(root_repo, entry.unwrap().path().to_str().unwrap()))
        .flatten()
        .collect()
}

#[tauri::command]
fn create_new_document(repo: String, title: &str) {
    fs::create_dir_all(&repo).unwrap();
    let mut index_md = File::create(repo + "/index.md").unwrap();
    let header: String = {
        if title.contains(':') {
            format!("---\ntitle: |\n\t{title}\n---")
        }
        else {
            format!("---\ntitle: {title}\n---")
        }
    };
    index_md.write(header.as_bytes()).unwrap();
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
                parse_config_file,
                fetch_projects,
                create_new_document,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}