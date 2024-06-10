import MainPage from "./web_components/main_page.ts";
import DocumentViewer from "./web_components/document-viewer.ts";

window.addEventListener("DocumentSelected", event => {
    document.querySelector("main-page")
        .replaceWith(new DocumentViewer(event.detail.path_to_document));
})

window.addEventListener("GoBackToMainPage", event => {
    document.querySelector("document-viewer")
        .replaceWith(new MainPage);
})
