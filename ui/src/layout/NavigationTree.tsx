import {useBookStructureStore} from "../structure/BookStructureStore.ts";
import {useChapterStore} from "../chapter/ChapterStore.ts";
import styled from "styled-components";
import type {ChangeEvent} from "react";
import type {ChapterStructure} from "../structure/BookStructure.ts";

const ChapterList = styled.ul`
    list-style: none;
    overflow-y: scroll;
    margin: 0;
    padding-left: 20px;
`;

const ChapterSummary = styled.summary`
    cursor: pointer;
    font-weight: 600;
`;

const SubChapterList = styled.ul`
    list-style: none;
    margin: 0;
    padding: 8px;
`;

const EditLink = styled.a<{ $active: boolean; }>`
    text-decoration: none;
    color: #111827;
    font-weight: ${props => props.$active ? 600 : 400};
`;

const Search = styled.input`
    margin: 9px;
    padding: 8px;
    border-radius: 4px;
    border: 1px solid #e5e7eb;
`;

const NavigationTreeDiv = styled.div`
    height: 100%;
    margin: 0px;
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
  useBookStructureStore.setState({chapterSearch: event.target.value});
}

function filterChapters(chapters: ChapterStructure[], rawChapterSearch: string | null): ChapterStructure[] {
  if (!rawChapterSearch) {
    return chapters;
  }
  let chapterSearch = rawChapterSearch.trim().toLowerCase();
  if (chapterSearch === "") {
    return chapters;
  }
  chapters = globalThis.structuredClone(chapters);
  for (let chapter of chapters) {
    chapter.chapters = chapter.chapters.filter((chapter) => {
      return chapter.label.toLowerCase().includes(chapterSearch);
    });
  }
  chapters = chapters.filter((chapter) => {
    return chapter.chapters.length > 0;
  });
  return chapters;

}


export function NavigationTree() {
  let chapters = useBookStructureStore((store) => store.book.chapters);
  let chapterSearch = useBookStructureStore((store) => store.chapterSearch);
  let chapter_id = useChapterStore(store => store.chapter_id);
  chapters = filterChapters(chapters, chapterSearch);
  return <NavigationTreeDiv>
    <Search type="text" placeholder="🔍 Search"
            onChange={changeChapterSearchParam}/>

    <ChapterList>
      {chapters.map(((chapter) =>
              <li key={chapter.id}>
                <ChapterSummary>{chapter.label}</ChapterSummary>
                <SubChapterList>
                  {chapter.chapters.map((chapter) =>
                      <li key={chapter.id}><EditLink href={`?chapter=${encodeURIComponent(chapter.id)}`}
                                                     $active={chapter_id === chapter.id}>{chapter.label}</EditLink>
                      </li>
                  )}

                </SubChapterList>
              </li>
      ))}
    </ChapterList>
  </NavigationTreeDiv>
}