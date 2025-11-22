import type {BookStructure, ChapterStructure} from "./BookStructure.ts";
import {createStore} from "../jestor/jestor.ts";

export interface BookStructureState {
  book: BookStructure,
  chapterSearch: string,
  chapterMap: Map<string, ChapterStructure>,
  //chapters: ChapterStructure[],
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


export const bookStructureStore = createStore({
  name: "Book Structure",
  initialState: {
    book: {
      title: "<loading>",
      chapters: [],
    },
    chapterSearch: "",
    chapterMap: new Map<string, ChapterStructure>(),
  } satisfies BookStructureState,
  actions: {
    reload() {
      (async () => {
        let response = await fetch("./api/structure.json");
        let book = await response.json() as BookStructure;

        bookStructureStore.dispatch.setBook(book);
      })();
    },
    setBook(state: BookStructureState, book: BookStructure) {
      state.book = book;
      state.chapterMap = createChapterMap(book);
    },
    setSearch(state: BookStructureState, search: string) {
      state.chapterSearch = search;
    },
  },
  derivedState: {
    chapters(state: BookStructureState) {
      return filterChapters(state.book.chapters, state.chapterSearch);
    },
  },
});

bookStructureStore.dispatch.reload();


function filterChapters(chapters: ChapterStructure[], rawChapterSearch: string | undefined): ChapterStructure[] {
  if (!rawChapterSearch) {
    return chapters;
  }
  let chapterSearch = rawChapterSearch.trim().toLowerCase();
  if (chapterSearch === "") {
    return chapters;
  }
  chapters = clone(chapters);
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

function clone<T>(value: T): T {
  return JSON.parse(JSON.stringify(value));
}