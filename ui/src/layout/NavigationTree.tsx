import {siteStore} from "../site/SiteStore.ts";
import {documentStore} from "../document/DocumentStore.ts";
import styled from "styled-components";
import type {ChangeEvent} from "react";

const ChapterList = styled.ul`
    list-style: none;
    overflow-y: scroll;
    margin: 0;
    padding-left: 20px;
`;

const EditLink = styled.a<{ $active: boolean; }>`
    text-decoration: none;
    color: #111827;
    font-weight: ${props => props.$active ? 600 : 400};
`;

const Search = styled.input`
    margin: 9px;
    padding: 8px;
    border-radius: 6px;
    border: 1px solid #e5e7eb;
`;

const NavigationTreeDiv = styled.div`
    height: 100%;
    margin: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;

    & > *:first-child {
        flex: 0 0 auto;
    }

    & > *:nth-child(2) {
        flex: 1 1 auto;
    }
`;

function changeChapterSearchParam(event: ChangeEvent<HTMLInputElement>) {
  siteStore.dispatch.setSearch(event.target.value);
}

export function NavigationTree() {
  let documentMap = siteStore.select.documentMap();
  let chapterSearch = siteStore.select.chapterSearch();
  let documentId = documentStore.select.document_id();
  return <NavigationTreeDiv>
    <Search type="text" placeholder="🔍 Search"
            value={chapterSearch}
            onChange={changeChapterSearchParam}/>

    <ChapterList>
      {Array.from(documentMap).map((([_key, document]) =>
              <li key={document.id}><EditLink href={`?document=${encodeURIComponent(document.id)}`}
                                              $active={documentId === document.id}>{document.title}</EditLink>
              </li>
      ))}
    </ChapterList>
  </NavigationTreeDiv>
}