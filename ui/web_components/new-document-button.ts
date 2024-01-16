// access the pre-bundled global API functions
import { invoke } from "@tauri-apps/api/tauri";

// TODO: Find out why the fuck does Tauri not allow deriving buttons
// THEN: Fix code to not reinvent the fucking `button` element…
export default class NewDocumentButton extends HTMLElement /* HTMLButtonElement */ {
    constructor(existing_repos) {
        super();
        const shadow = this.attachShadow({ mode: "open" });
        shadow.innerHTML = `
            <style>
                button#main {
                    aspect-ratio: 21 / 29.7;
                    width: 150px;
                    font-weight: bolder;
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
            const title = shadow.querySelector<HTMLInputElement>("input#title").value;
            const repo  = shadow.querySelector<HTMLInputElement>("input#repo").value;

            invoke('create_new_document', { title, repo }).then(path_to_document => {
                self.dispatchEvent(new CustomEvent("DocumentSelected", {
                    bubbles: true,
                    composed: true,
                    detail: { path_to_document },
                }));
            });
        });
    }
}
customElements.define("new-document-button", NewDocumentButton);
