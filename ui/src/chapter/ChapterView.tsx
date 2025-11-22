import {chapterStore} from "./ChapterStore.ts";
import {markdownToHtml} from "../util/markdown.ts";
import styled from "styled-components";

const Article = styled.article`
    max-width: 900px;
    margin: 0 auto;

    table {
        border-collapse: collapse;
    }

    table th,
    table td {
        border: 1px solid #ddd;
        padding: 0.5em;
    }
`;

export function ChapterView() {
  let chapter_markdown = chapterStore.select.markdown();
  let edit_url = chapterStore.select.edit_url();
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
