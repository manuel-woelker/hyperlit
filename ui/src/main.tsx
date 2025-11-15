import {StrictMode} from 'react'
import {createRoot} from 'react-dom/client'
import './index.css'
import {App} from "./App.tsx";
import {useBookStructureStore} from "./structure/BookStructureStore.ts";
import {useChapterStore} from "./chapter/ChapterStore.ts";

createRoot(document.getElementById('root')!).render(
    <StrictMode>
      <App/>
    </StrictMode>,
)


const evtSource = new EventSource("api/events", {});
evtSource.onmessage = (event) => {
  console.log(event);
  useBookStructureStore.getState().reload();
  useChapterStore.getState().update_from_url();
};

useBookStructureStore.subscribe(updateDocumentTitle);
useChapterStore.subscribe(updateDocumentTitle);


function updateDocumentTitle() {
  let bookStructure = useBookStructureStore.getState();
  let chapter_id = useChapterStore.getState().chapter_id;
  let chapter = bookStructure.chapterMap.get(chapter_id ?? " nada ");
  if (chapter) {
    document.title = `${bookStructure.book.title} - ${chapter.label}`
  } else {
    document.title = `${bookStructure.book.title}`
  }
}

