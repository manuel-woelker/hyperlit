import {StrictMode} from 'react'
import {createRoot} from 'react-dom/client'
import './index.css'
import {App} from "./App.tsx";
import {bookStructureStore} from "./structure/BookStructureStore.ts";
import {chapterStore} from "./chapter/ChapterStore.ts";

createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App/>
    </StrictMode>,
)


const evtSource = new EventSource("api/events", {});
evtSource.onmessage = (event) => {
  console.log(event);
  bookStructureStore.dispatch.reload();
  chapterStore.dispatch.update_from_url();
};

bookStructureStore.subscribe(updateDocumentTitle);
chapterStore.subscribe(updateDocumentTitle);


function updateDocumentTitle() {
  let bookStructure = bookStructureStore.getSnapshot();
  let chapter_id = chapterStore.getSnapshot().chapter_id;
  let chapter = bookStructure.chapterMap.get(chapter_id ?? " nada ");
  if (chapter) {
    document.title = `${bookStructure.book.title} - ${chapter.label}`
  } else {
    document.title = `${bookStructure.book.title}`
  }
}

