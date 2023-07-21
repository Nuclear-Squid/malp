// access the pre-bundled global API functions
const { invoke } = window.__TAURI__.tauri

/*
 * TODO: Find better names for half of the classes in this file
*/

class StateManager extends HTMLElement {
	constructor() {
		super();
		const shadow = this.attachShadow({ mode: "open" });
		shadow.innerHTML = `
			<style>
				h1 {
					text-align: center;
					color: #eee;
					font-size: 3em;
					margin: 1em;
				}
			</style>
			<h1> Markdown Live Preview </h1>
			<document-selector/>
		`;

		this.addEventListener("onDocumentSelected", event => {
			shadow.innerHTML = ``;
			shadow.appendChild(new DocumentViewer(event.detail.path_to_document));
		});
	}
}
customElements.define("state-manager", StateManager);


class DocumentViewer extends HTMLElement {
	constructor(documentPath) {
		super();
		const shadow = this.attachShadow({ mode: "open" });
		shadow.innerHTML = `<style></style> <div id="page" style="margin-top: 3em;"></div>`;
		invoke('load_document', { documentPath }).then(response => {
			shadow.querySelector("style").innerHTML = response.stylesheet;
			shadow.querySelector("#page").innerHTML = response.content;
		});
	}
}
customElements.define("document-viewer", DocumentViewer);


// TODO: Find out why the fuck does Tauri not allow deriving buttons
// THEN: Fix code to not reinvent the fucking `button` element…
class NewDocumentButton extends HTMLElement /* HTMLButtonElement */ {
	constructor(existing_repos) {
		super();
		const shadow = this.attachShadow({ mode: "open" });
		shadow.innerHTML = `
			<style> 
				button#main {
					aspect-ratio: 21 / 29.7;
					width: 150px;
					font-weight: bold;
				}

				dialog {
					background-color: #333;
					border: 3px solid orange;
					border-radius: 30px;
					color: #eee;
				}
			</style>

			<button id="main"> + </button>

			<dialog>
				<div style="display: flex; flex-direction: column">
					<h1> New Document </h1>
					<span> Repo:  <input id="repo" type="text" list="known-repos"></input> </span>
					<span> Title: <input id="title"></input> </span>
					<span>
						<button id="create"> create </button>
						<button id="close"> close </button>
					</span>
					<datalist id="known-repos"></datalist>
				</div>
			</dialog>
		`;

		const dialog = shadow.querySelector("dialog");
		dialog.querySelector("datalist").innerHTML =
			existing_repos.map(repo => `<option value="${repo}">`).join('');

		shadow.querySelector("#main").addEventListener("click", () => {
			dialog.show();
		})
 
		shadow.querySelector("#close").addEventListener("click", () => {
			dialog.close();
		});

		// Keep a reference to the web-component;
		const self = this;
		shadow.querySelector("#create").addEventListener("click", () => {
			const title = shadow.querySelector("input#title").value;
			const repo = shadow.querySelector("input#repo").value + title;
			invoke('create_new_document', { title, repo }).then(path_to_document => {
				console.log(path_to_document);
				self.dispatchEvent(new CustomEvent("onDocumentSelected", {
					bubbles: true,
					composed: true,
					detail: { path_to_document },
				}));
			});
		});
	}
}
customElements.define("new-document-button", NewDocumentButton);


class DocumentHandle extends HTMLElement {
	constructor(name, parent_dir_path) {
		super();
		this.name = name;
		this.parent_dir_path  = parent_dir_path;
		const shadow = this.attachShadow({ mode: "open" });
		shadow.innerHTML = `
			<style> 
				button {
					aspect-ratio: 21 / 29.7;
					width: 150px;
				}
			</style>
			<button> ${name} </button>
		`;
		// Using `function` to not fuck `this` up.
		shadow.querySelector("button").addEventListener("click", function () {
			this.dispatchEvent(new CustomEvent("onDocumentSelected", {
				bubbles: true,
				composed: true,
				detail: { path_to_document: parent_dir_path + name },
			}));
		});
	}
}
customElements.define("document-handle", DocumentHandle);


function remove_consecutive_duplicates(array) {
	if (array == []) return [];
	let rv = [array[0]];
	// iterate over indexes because array.slice(1) would copy the array
	for (let i = 1; i < array.length; i++) {
		if (array[i] != rv.at(-1)) {
			rv.push(array[i])
		}
	}
	return rv;
}


class DocumentSelector extends HTMLElement {
	constructor() {
		super();
		const shadow = this.attachShadow({ mode: "open" });
		shadow.innerHTML = `<hr>`;
		invoke('fetch_projects', {}).then(projects => {
			// Extracting all of the repos in base, removing duplicates
			const repos = projects.map( ({ parent_dir_path }) => parent_dir_path);
			// repos are garentied to be alphabetically ordered
			shadow.appendChild(new NewDocumentButton(remove_consecutive_duplicates(repos)));

			projects.forEach(({ name, parent_dir_path }) => {
				shadow.appendChild(new DocumentHandle(name, parent_dir_path))
			});
		})
	}
}
customElements.define("document-selector", DocumentSelector);
