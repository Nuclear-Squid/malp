// access the pre-bundled global API functions
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";
import { Previewer } from "pagedjs";

interface Payload {
    stylesheet: string,
    content: string,
}

export default class DocumentViewer extends HTMLElement {
    constructor(documentPath) {
        super();
        const shadow = this.attachShadow({ mode: "open" });
        const go_back_button = document.createElement("button");
        const print_button = document.createElement("button");
        const doc = document.createElement("div");

        go_back_button.innerText = "Go back to main page";
        go_back_button.onclick = () => {
            this.dispatchEvent(new CustomEvent("GoBackToMainPage", {
                bubbles: true,
                composed: true,
            }));
        };
        print_button.innerText = "print";
        print_button.onclick = () => {
            window.print()
        };
        doc.style.cssText = "margin-top: 3em; background-color: white; height: 100%; width: 100%";

        shadow.appendChild(go_back_button);
        shadow.appendChild(print_button);
        shadow.appendChild(doc);

        const render_document = (new_doc: Payload) => {
            const paged = new Previewer();
            // paged.preview(new_doc.content, [], document.body);
            paged.preview(new_doc.content, [], doc);
        };

        // Render document on initial load
        invoke<Payload>('load_document', { documentPath }).then(render_document);

        // Rerender document when document is modified
        listen<{ message: Payload }>("document-modified", payload => render_document(payload.message))
    }
}
customElements.define("document-viewer", DocumentViewer);
