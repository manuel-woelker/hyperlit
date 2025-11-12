import {create, type UseBoundStore} from "zustand/react";
import type {StoreApi} from "zustand/vanilla";


export interface ChapterStore {
  chapter_id: string | null,
  update_from_url: () => void,
}

export const useChapterStore: UseBoundStore<StoreApi<ChapterStore>> = create((set) => ({
  chapter_id: null,
  update_from_url: () => {
    let url = new URL(window.location.href);
    let chapter_id = url.searchParams.get("chapter");
    set((state: ChapterStore) => {
      return {
        ...state,
        chapter_id,
      }
    });
  }
}));

useChapterStore.getState().update_from_url();

document.addEventListener('click', function (event) {
  const link = (event.target as Element)?.closest('a');
  if (!link) return;

  event.preventDefault();
  window.history.pushState({}, '', link.href);

  // Update page content
  console.log('Navigated to:', link.href);
  useChapterStore.getState().update_from_url();
});

// Handle back/forward navigation too
window.addEventListener('popstate', () => {
  console.log('User navigated with Back/Forward. Page:', window.location.href);
  useChapterStore.getState().update_from_url();
});