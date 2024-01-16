import DocumentSelector from "./web_components/document-selector.ts"
import DocumentViewer   from "./web_components/document-viewer.ts"

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

        this.addEventListener("DocumentSelected", event => {
            shadow.innerHTML = ``;
            shadow.appendChild(new DocumentViewer((<any>event).detail.path_to_document));
        });
    }
}
customElements.define("app-root", StateManager);
