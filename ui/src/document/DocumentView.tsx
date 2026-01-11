import {documentStore} from "./DocumentStore.ts";
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

export function DocumentView() {
  let chapter_markdown = documentStore.select.markdown();
  let edit_url = documentStore.select.edit_url();
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
