import type {DocumentInfo, SiteInfo} from "./SiteInfo.ts";
import {createStore} from "../jestor/jestor.ts";

export interface SiteState {
  title: string,
  titleSearch: string,
  documentMap: Map<string, DocumentInfo>,
  documents: DocumentInfo[],
}

export const siteStore = createStore({
  name: "Book Structure",
  initialState: {
    title: "<loading>",
    titleSearch: "",
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
      state.titleSearch = search;
    },
  },
  derivedState: {
    filteredDocuments(state: SiteState) {
      return filterDocuments(state.documents, state.titleSearch);
    }
  },
});

siteStore.dispatch.reload();

function filterDocuments(documents: DocumentInfo[], rawTitleSearch: string): DocumentInfo[] {
  if (!rawTitleSearch) {
    return documents;
  }
  let titleSearch = rawTitleSearch.trim().toLowerCase();
  if (titleSearch === "") {
    return documents;
  }
  documents = documents.filter((document) => {
    return document.title.toLowerCase().includes(titleSearch);
  });
  return documents;
}

