import {marked} from 'marked';
import {useChapterStore} from "./ChapterStore.ts";
import {useEffect, useState} from "react";

export function ChapterView() {
  let chapter_markdown = useChapterStore(store => store.markdown);
  let [html, setHtml] = useState('Loading...');
  useEffect(() => {
    if (!chapter_markdown) {
      // Debounce the transition
      const t = setTimeout(() => {
        setHtml("Loading...");
      }, 200); // delay in ms

      return () => clearTimeout(t);
    } else {
      setHtml(marked.parse(chapter_markdown, {async: false}));
    }
  }, [chapter_markdown]);
  return <article style={{maxWidth: 900, margin: '0 auto'}} dangerouslySetInnerHTML={{__html: html}}>
  </article>
}
