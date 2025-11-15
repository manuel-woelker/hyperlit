import {create, type UseBoundStore} from "zustand/react";
import type {BookStructure, ChapterStructure} from "./BookStructure.ts";
import type {StoreApi} from "zustand/vanilla";


export interface BookStructureStore {
  book: BookStructure,
  chapterMap: Map<string, ChapterStructure>,
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


export const useBookStructureStore: UseBoundStore<StoreApi<BookStructureStore>> = create((set) => ({
  book: {
    title: "<loading>",
    chapters: [],
  },
  chapterMap: new Map<string, ChapterStructure>(),
  reload: () => {
    (async () => {
      let response = await fetch("./api/structure.json");
      let book = await response.json() as BookStructure;

      let chapterMap = createChapterMap(book);
      set((state) => {
        return {
          ...state,
          book,
          chapterMap,
        };
      });
    })();
  }
}));

useBookStructureStore.getState().reload();