import {useChapterStore} from "./ChapterStore.ts";
import {markdownToHtml} from "../util/markdown.ts";

export function ChapterView() {
  let chapter_markdown = useChapterStore(store => store.markdown);
  let html = "Loading..."
  if (chapter_markdown) {
    html = markdownToHtml(chapter_markdown);
  }
  return <article style={{maxWidth: 900, margin: '0 auto'}} dangerouslySetInnerHTML={{__html: html}}>
  </article>
}
