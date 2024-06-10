// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

/*
 * TODO: Refactor code to have real error handling
 */

use std::{
    path::{ Path, PathBuf },
    fs::{ self, File },
    io::Write,
    sync::OnceLock,
};

use serde::{ Serialize, Deserialize };
use walkdir::WalkDir;
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

macro_rules! config {
    ($field: ident) => (&MALP_CONFIG.get().unwrap().$field)
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
fn fetch_projects() -> Vec<PathBuf> {
    let root_repo = config!(documents_root_repo);
    WalkDir::new(root_repo)
        .into_iter()
        .filter_map(|e| e.ok())  // Ignore errors
        .filter(|e| e.file_name().to_str() == Some("index.md"))
        .map(|e| e.path().parent().unwrap().strip_prefix(root_repo).unwrap().to_owned())
        .collect()
}

/// Returns the path to the newly created document.
#[tauri::command]
fn create_new_document(repo: &str, title: &str) -> PathBuf {
    let mut path_to_document: PathBuf = config!(documents_root_repo).clone();
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
    let mut absolute_path = config!(documents_root_repo).clone();
    absolute_path.push(document_path);
    load_document_absolute(absolute_path)
}

fn load_document_absolute(mut document_path: PathBuf) -> DocumentContents {
    use pandoc::*;
    document_path.push("index.md");
    let markdown_extentions: Vec<MarkdownExtension> = vec![];
    let raw_pandoc_response =
        pandoc::new()
            .set_input(InputKind::Files(vec![document_path.clone()]))
            .set_input_format(InputFormat::Markdown, markdown_extentions.clone())
            .add_option(PandocOption::Standalone)
            .set_output_format(OutputFormat::Html, markdown_extentions)
            .set_output(OutputKind::Pipe)
            .clone().execute();

    let pandoc_response = match raw_pandoc_response {
        Ok(PandocOutput::ToBuffer(response)) => response,
        Err(e) => panic!("Pandoc Failed: {e}"),
        Ok(_) => panic!("Enexpected pandoc output"),
    };

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
