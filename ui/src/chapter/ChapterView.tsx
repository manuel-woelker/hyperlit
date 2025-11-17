import {useChapterStore} from "./ChapterStore.ts";
import {markdownToHtml} from "../util/markdown.ts";
import styled from "styled-components";

const Article = styled.article`
    max-width: 900px;
    margin: 0 auto;
`;

export function ChapterView() {
  let chapter_markdown = useChapterStore(store => store.markdown);
  let edit_url = useChapterStore(store => store.edit_url);
  let html = "Loading..."
  if (chapter_markdown) {
    html = markdownToHtml(chapter_markdown);
  }
  return <div>
    <a href={edit_url ?? "#"}>Edit</a>
    <Article dangerouslySetInnerHTML={{__html: html}}>
    </Article>
  </div>;
}
