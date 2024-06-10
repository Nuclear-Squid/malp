// access the pre-bundled global API functions
import { invoke } from "@tauri-apps/api/tauri";

import NewDocumentButton from "./new-document-button"
import DocumentHandle    from "./document-handle"

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
    constructor(folderName) {
        super();

        const shadow = this.attachShadow({ mode: "open" });
        shadow.innerHTML = `<h2> ${folderName} </h2>`;
        this.id = folderName;
    }

    pushProject(projectName) {
        this.shadowRoot.appendChild(new DocumentHandle(projectName, this.id))
    }
}
customElements.define("document-selector", DocumentSelector);
