import {create, type UseBoundStore} from "zustand/react";
import type {BookStructure, ChapterStructure} from "./BookStructure.ts";
import type {StoreApi} from "zustand/vanilla";
import {immer} from "zustand/middleware/immer";


export interface BookStructureStore {
  book: BookStructure,
  chapterSearch: string | undefined,
  chapterMap: Map<string, ChapterStructure>,
  chapters: ChapterStructure[],
  reload: () => void,
}

function createChapterMap(book: BookStructure) {
  let map = new Map<string, ChapterStructure>();

  function addChaptersToMap(chapters: ChapterStructure[]) {
    for (let chapter of chapters) {
      map.set(chapter.id, chapter);
      addChaptersToMap(chapter.chapters)
    }
  }

  addChaptersToMap(book.chapters);

  return map;
}


export const useBookStructureStore: UseBoundStore<StoreApi<BookStructureStore>> = create(immer(set => ({
  book: {
    title: "<loading>",
    chapters: [],
  },
  chapters: [],
  chapterSearch: undefined,
  chapterMap: new Map<string, ChapterStructure>(),
  reload: () => {
    (async () => {
      let response = await fetch("./api/structure.json");
      let book = await response.json() as BookStructure;

      let chapterMap = createChapterMap(book);
      set((state) => {
        state.book = book;
        state.chapterMap = chapterMap;
        state.chapters = filterChapters(book.chapters, state.chapterSearch);
      });
    })();
  },
})));

useBookStructureStore.getState().reload();
useBookStructureStore.subscribe((state, prevState) => {
  if (state.chapterSearch !== prevState.chapterSearch) {
    useBookStructureStore.setState({chapters: filterChapters(state.book.chapters, state.chapterSearch)});
  }
});


function filterChapters(chapters: ChapterStructure[], rawChapterSearch: string | undefined): ChapterStructure[] {
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
