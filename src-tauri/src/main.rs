// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/*
 * TODO: Refactor code to have real error handling
 */

use std::path::{ Path, PathBuf };
use std::fs::{ self, File };
use std::io::Write;
use std::sync::OnceLock;
use serde::{ Serialize, Deserialize };
use notify::Watcher;
use tauri::Manager;

/// A struct (singleton) representing the global config of the app.
/// This config file is located at `~/.config/mapl_conf.toml`.
#[derive(Debug, Serialize, Deserialize)]
struct Config {
    documents_root_repo: String,
}

/// The global handle for the config, it’s filled in at run-time in `main`.
static MALP_CONFIG: OnceLock<Config> = OnceLock::new();

/// A struct representing a handle to a document in the file system.
#[derive(Serialize, Deserialize)]
struct DocumentDescriptor {
    /// Name of the document
    name: String,
    /// Path to the dir containing the document (relative to `documents_root_repo`)
    parent_dir_path: String,
}

/// A struct representing a document being rendered.
#[derive(Clone, Serialize, Deserialize)]
struct DocumentContents {
    /// contents of the `style.css` file of the project.
    stylesheet: String,
    /// contents of the `index.md` file of the project.
    content: String,
}

/// Struct send along with events from backend to frontend.
#[derive(Clone, Serialize)]
struct Payload {
    message: DocumentContents,
}

/// Fetches all of the documents beneath `documents_root_repo`.
#[tauri::command]
fn fetch_projects() -> Vec<DocumentDescriptor> {
    let root_repo = &MALP_CONFIG.get().unwrap().documents_root_repo;
    fetch_projects_inner(root_repo, root_repo)
}

fn fetch_projects_inner(root_repo: &str, current_dir: &str) -> Vec<DocumentDescriptor> {
    // Check if current dir is a document, and return it if it is
    for entry in fs::read_dir(current_dir).unwrap() {
        let Ok(entry) = entry else { continue; };

        if entry.file_name() == "index.md" {
            let (absolute_parent_dir_path, project_name) =
                current_dir.rsplit_once('/').unwrap();
            let parent_dir_path = absolute_parent_dir_path
                    .strip_prefix(root_repo)
                    .unwrap();

            return vec![DocumentDescriptor {
                name: project_name.to_owned(),
                parent_dir_path: parent_dir_path.to_owned() + "/",
            }];
        }
    }

    // Current dir isn’t a document, check sub directories
    fs::read_dir(current_dir).unwrap()
        .map(|entry| fetch_projects_inner(root_repo, entry.unwrap().path().to_str().unwrap()))
        .flatten()
        .collect()
}

/// Returns the path to the newly created document.
#[tauri::command]
fn create_new_document(mut repo: String, title: &str) -> String {
    let rv_document_path = repo.clone();
    repo = MALP_CONFIG.get().unwrap().documents_root_repo.clone() + &repo;
    fs::create_dir_all(&repo).unwrap();
    let mut index_md = File::create(repo.clone() + "/index.md").unwrap();
    let header: String = {
        if title.contains(':') {
            format!("---\ntitle: |\n\t{title}\n---")
        }
        else {
            format!("---\ntitle: {title}\n---")
        }
    };
    index_md.write(header.as_bytes()).unwrap();

    let mut stylesheet = File::create(repo.clone() + "/stylesheet.css").unwrap();
    stylesheet.write("#page { padding: 3em }".as_bytes()).unwrap();

    return rv_document_path;
}

#[tauri::command]
fn load_document(document_path: &str) -> DocumentContents {
    let parent_dir_absolute_path =
        MALP_CONFIG.get().unwrap().documents_root_repo.clone() + document_path;
    load_document_absolute(parent_dir_absolute_path.into())
}

fn load_document_absolute(parent_dir_absolute_path: PathBuf) -> DocumentContents {
    // move variable out to rename it
    let mut markdown_file_path = parent_dir_absolute_path.clone();
    markdown_file_path.push("index.md");

    use pandoc::*;
    let markdown_extentions: Vec<MarkdownExtension> = vec![];
    let Ok(PandocOutput::ToBuffer(pandoc_response)) = 
        pandoc::new()
            .set_input(InputKind::Files(vec![markdown_file_path]))
            .set_input_format(InputFormat::Markdown, markdown_extentions.clone())
            .add_option(PandocOption::Standalone)
            .set_output_format(OutputFormat::Html, markdown_extentions)
            .set_output(OutputKind::Pipe)
            .clone().execute()
        else { panic!("no") };

    let mut stylesheet_path = parent_dir_absolute_path;
    stylesheet_path.push("stylesheet.css");
    let stylesheet_content = fs::read_to_string(stylesheet_path).unwrap();

    DocumentContents {
        stylesheet: stylesheet_content,
        content: pandoc_response
    }
}

fn parse_config_file() -> Config {
    let config_file_path = shellexpand::tilde("~/.config/malp_conf.toml");
    let config_contents = fs::read_to_string(config_file_path.as_ref()).unwrap();
    let mut rv: Config = toml::from_str(&config_contents).unwrap();
    rv.documents_root_repo = shellexpand::tilde(&rv.documents_root_repo).as_ref().to_owned();
    rv
}

fn main() {
    MALP_CONFIG.set(parse_config_file()).unwrap();

    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
                fetch_projects,
                create_new_document,
                load_document,
        ])
        .build(tauri::generate_context!())
        .expect("cannot build application");

    let app_handle = app.handle();

    use notify::event as NE;
    let mut document_folder_watcher =
        notify::recommended_watcher(move |res: notify::Result<NE::Event>| {
            let Ok(mut res) = res else { return () };
            if matches!(res.kind, NE::EventKind::Modify(NE::ModifyKind::Data(_))) {
                let mut document_path = res.paths.remove(0);
                document_path.pop();  // get rid of modified file name
                app_handle.emit_all("document-modified", Payload {
                    message: load_document_absolute(document_path),
                }).unwrap();
            }
        })
    .expect("eror creating the document folder watcher");

    document_folder_watcher
        .watch(Path::new(&MALP_CONFIG.get().unwrap().documents_root_repo), notify::RecursiveMode::Recursive)
        .expect("error watching folder");

    app.run(|_app_handle, event| match event {
        tauri::RunEvent::ExitRequested { api, .. } => {
            api.prevent_exit();
        },
        _ => {},
    })
}
