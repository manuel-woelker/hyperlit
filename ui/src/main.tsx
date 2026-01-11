import {StrictMode} from 'react'
import {createRoot} from 'react-dom/client'
import './index.css'
import {App} from "./App.tsx";
import {siteStore} from "./site/SiteStore.ts";
import {documentStore} from "./document/DocumentStore.ts";

createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App/>
    </StrictMode>,
)


const evtSource = new EventSource("api/events", {});
evtSource.onmessage = (event) => {
  console.log(event);
  siteStore.dispatch.reload();
  documentStore.dispatch.update_from_url();
};

siteStore.subscribe(updateDocumentTitle);
documentStore.subscribe(updateDocumentTitle);


function updateDocumentTitle() {
  let siteState = siteStore.getSnapshot();
  let documentId = documentStore.getSnapshot().document_id;
  let documentInfo = siteState.documentMap.get(documentId ?? " nada ");
  if (documentInfo) {
    document.title = `${siteState.title} - ${documentInfo.title}`
  } else {
    document.title = `${siteState.title}`
  }
}

