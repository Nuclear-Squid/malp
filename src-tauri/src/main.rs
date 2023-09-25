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
    documents_root_repo: PathBuf,
}

/// The global handle for the config, it’s filled in at run-time in `main`.
static MALP_CONFIG: OnceLock<Config> = OnceLock::new();

macro_rules! get_in_config {
    ($field :  ident) => {
        MALP_CONFIG.get().unwrap().$field.clone()
    }
}

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
    let root_repo: Box<str> =
            get_in_config!(documents_root_repo).to_str().unwrap().into();
    fetch_projects_inner(&root_repo, &root_repo)
}

fn fetch_projects_inner(root_repo: &str, current_dir: &str) -> Vec<DocumentDescriptor> {
    // Check if current dir is a document, and return it if it is
    for entry in fs::read_dir(current_dir).unwrap() {
        let Ok(entry) = entry else { continue; };

        if entry.file_name() == "index.md" {
            let (absolute_parent_dir_path, project_name) =
                current_dir.rsplit_once('/').unwrap();

            let parent_dir_path: String = unsafe {
                let tmp = absolute_parent_dir_path.strip_prefix(root_repo).unwrap();
                    // .to_string() + "/"
                if tmp == "" {
                    "/".to_string()
                } else {
                    tmp.get_unchecked(1..).to_string() + "/"
                }
            };

            return vec![DocumentDescriptor {
                name: project_name.to_owned(),
                parent_dir_path: parent_dir_path,
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
fn create_new_document(repo: &str, title: &str) -> PathBuf {
    let mut path_to_document: PathBuf = get_in_config!(documents_root_repo);
    path_to_document.push(repo);
    path_to_document.push(title);
    fs::create_dir_all(&path_to_document).unwrap();

    let header: String = {
        if title.contains(':') {
            format!("---\ntitle: |\n\t{title}\n---")
        } else {
            format!("---\ntitle: {title}\n---")
        }
    };

    path_to_document.push("index.md");
    let mut index_md = File::create(&path_to_document).unwrap();
    index_md.write_all(header.as_bytes()).unwrap();

    path_to_document.set_file_name("stylesheet.css");
    let mut stylesheet = File::create(&path_to_document).unwrap();
    stylesheet.write(b"#page { padding: 3em }").unwrap();

    path_to_document.pop();
    path_to_document
}

#[tauri::command]
fn load_document(document_path: &str) -> DocumentContents {
    let mut absolute_path = get_in_config!(documents_root_repo);
    absolute_path.push(document_path);
    load_document_absolute(absolute_path)
}

fn load_document_absolute(mut document_path: PathBuf) -> DocumentContents {
    use pandoc::*;
    document_path.push("index.md");
    let markdown_extentions: Vec<MarkdownExtension> = vec![];
    let Ok(PandocOutput::ToBuffer(pandoc_response)) = 
        pandoc::new()
            .set_input(InputKind::Files(vec![document_path.clone()]))
            .set_input_format(InputFormat::Markdown, markdown_extentions.clone())
            .add_option(PandocOption::Standalone)
            .set_output_format(OutputFormat::Html, markdown_extentions)
            .set_output(OutputKind::Pipe)
            .clone().execute()
        else { panic!("no") };

    document_path.set_file_name("stylesheet.css");
    DocumentContents {
        stylesheet: fs::read_to_string(document_path).unwrap(),
        content   : pandoc_response,
    }
}

fn parse_config_file() -> Config {
    let config_file_path = shellexpand::tilde("~/.config/malp_conf.toml");
    let config_contents = fs::read_to_string(config_file_path.as_ref()).unwrap();
    let mut rv: Config = toml::from_str(&config_contents).unwrap();
    rv.documents_root_repo = shellexpand::tilde(rv.documents_root_repo.to_str().unwrap()).as_ref().into();
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
