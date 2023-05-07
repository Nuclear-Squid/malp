// access the pre-bundled global API functions
const { invoke } = window.__TAURI__.tauri

async function load_main_page() {
	const malp_config = await invoke('parse_config_file', {});
	const projects = await invoke('fetch_projects', {
		rootRepo:   malp_config.documents_root_repo,
		currentDir: malp_config.documents_root_repo
	});

	const projects_dom_section = document.querySelector("section#projects");
	const new_document_dialog = document.querySelector("dialog#new-document");
	const data_list_repos = new_document_dialog.querySelector("datalist");

	let previous_dir = "";
	for (const { name, parent_dir_path } of projects) {
		projects_dom_section.innerHTML += `<button class="project"> ${name} </button>`;

		if (parent_dir_path != previous_dir) {
			previous_dir = parent_dir_path;
			data_list_repos.innerHTML += `<option value="${parent_dir_path}">`;
		}
	}

	document.querySelector("button#new-document-button")
		.addEventListener("click", _event => {
			new_document_dialog.show();
		});

	new_document_dialog.querySelector("button#close")
		.addEventListener("click", _event => { new_document_dialog.close() });

	new_document_dialog.querySelector("button#create")
		.addEventListener("click", _event => {
			const title = new_document_dialog.querySelector("input#title").value;
			const repo = [
				malp_config.documents_root_repo,
				new_document_dialog.querySelector("input#repo").value,
				title
			].join('')

			invoke('create_new_document', { title, repo, });
		});
}

load_main_page();
