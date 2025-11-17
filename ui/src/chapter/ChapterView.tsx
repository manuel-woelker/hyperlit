import {useChapterStore} from "./ChapterStore.ts";
import {markdownToHtml} from "../util/markdown.ts";

export function ChapterView() {
  let chapter_markdown = useChapterStore(store => store.markdown);
  let edit_url = useChapterStore(store => store.edit_url);
  let html = "Loading..."
  if (chapter_markdown) {
    html = markdownToHtml(chapter_markdown);
  }
  return <div>
    <a href={edit_url ?? "#"}>Edit</a>
    <article style={{maxWidth: 900, margin: '0 auto'}} dangerouslySetInnerHTML={{__html: html}}>
    </article>
  </div>;
}
