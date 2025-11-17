import {useBookStructureStore} from "../structure/BookStructureStore.ts";
import {useChapterStore} from "../chapter/ChapterStore.ts";
import styled from "styled-components";

const ChapterList = styled.ul`
    list-style: none;
    margin: 0;
    padding: 0;
`;

const ChapterSummary = styled.summary`
    cursor: pointer;
    font-weight: 600;
`;

const SubChapterList = styled.ul`
    list-style: none;
    margin: 0;
    padding: 0;
`;

const EditLink = styled.a<{ $active: boolean; }>`
    text-decoration: none;
    color: #111827;
    font-weight: ${props => props.$active ? 600 : 400};
`;

export function NavigationTree() {
  let chapters = useBookStructureStore((store) => store.book.chapters);
  let chapter_id = useChapterStore(store => store.chapter_id);
  return <ChapterList>
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
}