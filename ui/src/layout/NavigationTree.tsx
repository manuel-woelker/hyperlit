import {useBookStructureStore} from "../structure/BookStructureStore.ts";
import {useChapterStore} from "../chapter/ChapterStore.ts";

export function NavigationTree() {
  let chapters = useBookStructureStore((store) => store.book.chapters);
  let chapter_id = useChapterStore(store => store.chapter_id);
  return <ul style={{listStyle: 'none', margin: 0, padding: 0}}>
    {chapters.map(((chapter) =>
            <li key={chapter.id}>
              <summary style={{cursor: 'pointer', fontWeight: 600}}>{chapter.label}</summary>
              <ul style={{listStyle: 'none', margin: '8px 0 0 12px', padding: 0}}>
                {chapter.chapters.map((chapter) =>
                    <li key={chapter.id}><a href={`?chapter=${chapter.id}`}
                                            style={{
                                              textDecoration: 'none',
                                              color: '#111827',
                                              fontWeight: chapter_id === chapter.id ? 600 : 400,
                                            }}>{chapter.label}</a>
                    </li>
                )}

              </ul>
            </li>
    ))}
  </ul>
}