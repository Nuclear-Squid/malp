// access the pre-bundled global API functions
import { invoke } from "@tauri-apps/api/tauri";

import NewDocumentButton from "./new-document-button.ts"
import DocumentHandle    from "./document-handle.ts"

interface Payload {
    name: string,
    parent_dir_path: string,
}


function remove_consecutive_duplicates(array: Array<string>) {
    if (array.length == 0) return [];

    let rv = [array[0]];
    // iterate over indexes because array.slice(1) would copy the array
    for (let i = 1; i < array.length; i++) {
        if (array[i] != rv.at(-1)) {
            rv.push(array[i])
        }
    }
    return rv;
}


export default class DocumentSelector extends HTMLElement {
    constructor() {
        super();

        const shadow = this.attachShadow({ mode: "open" });
        shadow.innerHTML = `<hr>`;

        console.log("prout");

        invoke<Array<Payload>>('fetch_projects', {}).then(projects => {
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
