// access the pre-bundled global API functions
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";

interface Payload {
    stylesheet: string,
    content: string,
}

export default class DocumentViewer extends HTMLElement {
    constructor(documentPath) {
        super();
        const shadow = this.attachShadow({ mode: "open" });
        const style = document.createElement("style");
        const page  = document.createElement("div");

        page.id = "page";
        page.style.cssText = "margin-top: 3em;";

        shadow.appendChild(style);
        shadow.appendChild(page);

        invoke<Payload>('load_document', { documentPath }).then(response => {
            style.innerHTML = response.stylesheet;
            page .innerHTML = response.content;
        });

        listen<{ message: Payload }>("document-modified", ({ payload: { message } }) => {
            style.innerHTML = message.stylesheet;
            page .innerHTML = message.content;
        });
    }
}
customElements.define("document-viewer", DocumentViewer);
