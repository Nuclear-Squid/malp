import { invoke } from "@tauri-apps/api/tauri";

import DocumentSelector from "./document-selector"
import DocumentHandle   from "./document-handle"

export default class MainPage extends HTMLElement {
    constructor() {
        super();
        // const anchor = document.getElementById("documents-anchor");
        const shadow = this.attachShadow({ mode: "closed" });
        invoke<Array<string>>("fetch_projects").then(projects => {
            projects.map(filePath => {
                const pathElements = filePath.split('/');
                const folder = pathElements.slice(0, -1).join("/");
                const projectName = pathElements[pathElements.length - 1];
                return [folder, projectName];
            })
            .sort(([folder1], [folder2]) => folder1 > folder2)
            .forEach(([folder, projectName]) => {
                const wrapper = shadow.getElementById(folder) ?? shadow.appendChild(new DocumentSelector(folder));
                wrapper.pushProject(projectName)
            })
        });
    }
}
customElements.define("main-page", MainPage);
