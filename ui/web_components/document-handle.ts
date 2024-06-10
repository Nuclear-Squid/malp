export default class DocumentHandle extends HTMLElement {
    name: string;
    parent_dir_path: string;

    constructor(name, parent_dir_path) {
        super();
        this.name = name;
        this.parent_dir_path  = parent_dir_path;

        const shadow = this.attachShadow({ mode: "closed" });
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
            this.dispatchEvent(new CustomEvent("DocumentSelected", {
                bubbles: true,
                composed: true,
                detail: { path_to_document: parent_dir_path + '/' + name },
            }));
        });
    }
}
customElements.define("document-handle", DocumentHandle);
