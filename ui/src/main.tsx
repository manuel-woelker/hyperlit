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
