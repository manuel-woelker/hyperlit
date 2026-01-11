import type {DocumentInfo, SiteInfo} from "./SiteInfo.ts";
import {createStore} from "../jestor/jestor.ts";

export interface SiteState {
  title: string,
  chapterSearch: string,
  documentMap: Map<string, DocumentInfo>,
  documents: DocumentInfo[],
}

export const siteStore = createStore({
  name: "Book Structure",
  initialState: {
    title: "<loading>",
    chapterSearch: "",
    documentMap: new Map<string, DocumentInfo>(),
    documents: [],
  } satisfies SiteState,
  actions: {
    reload() {
      (async () => {
        let response = await fetch("./api/document-infos.json");
        let siteInfo = await response.json() as SiteInfo;
        siteStore.dispatch.setSiteInfo(siteInfo);
      })();
    },
    setSiteInfo(state: SiteState, siteInfo: SiteInfo) {
      state.documents = siteInfo.documents;
      state.title = siteInfo.title;
      state.documentMap = new Map(siteInfo.documents.map((documentInfo) => [documentInfo.id, documentInfo]));
    },
    setSearch(state: SiteState, search: string) {
      state.chapterSearch = search;
    },
  },
  derivedState: {
    /*    chapters(state: SiteState) {
          return filterChapters(state.book.chapters, state.chapterSearch);
        },*/
  },
});

siteStore.dispatch.reload();

/*
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
*/
