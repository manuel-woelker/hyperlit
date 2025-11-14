import {create, type UseBoundStore} from "zustand/react";
import type {BookStructure} from "./BookStructure.ts";
import type {StoreApi} from "zustand/vanilla";


export interface BookStructureStore {
  book: BookStructure,
  reload: () => void,
}

export const useBookStructureStore: UseBoundStore<StoreApi<BookStructureStore>> = create((set) => ({
  book: {
    title: "<loading>",
    chapters: [],
  },
  reload: () => {
    (async () => {
      let response = await fetch("./api/structure.json");
      let response_json = await response.json();
      document.title = response_json.title;
      set((state) => {
        return {
          ...state,
          book: response_json,
        };
      });
    })();
  }
}));

useBookStructureStore.getState().reload();