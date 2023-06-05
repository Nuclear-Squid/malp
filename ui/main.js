// access the pre-bundled global API functions
const { invoke } = window.__TAURI__.tauri

async function load_main_page() {
	const projects = await invoke('fetch_projects', {});

	const projects_dom_section = document.querySelector("section#projects");
	const new_document_dialog = document.querySelector("dialog#new-document");
	const data_list_repos = new_document_dialog.querySelector("datalist");

	let previous_dir = "";
	for (const { name, parent_dir_path } of projects) {
		const new_project_icon = document.createElement("button");
		new_project_icon.innerHTML = name;
		new_project_icon.classList.add("project");
		new_project_icon.addEventListener("click", (_event) => {
			const documentRelativePath = parent_dir_path + name + "/index.md";
			invoke('load_document', { documentRelativePath })
				.then(response => console.log(response));
		})
		projects_dom_section.appendChild(new_project_icon);
		// projects_dom_section.innerHTML += `<button class="project"> ${name} </button>`;

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
			const repo = new_document_dialog.querySelector("input#repo").value + title;
			invoke('create_new_document', { title, repo });
		});
}

load_main_page();
