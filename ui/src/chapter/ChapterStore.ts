import {create, type UseBoundStore} from "zustand/react";
import type {StoreApi} from "zustand/vanilla";
import {immer} from 'zustand/middleware/immer'

const LoadingStates = {
  Loading: "Loading",
  Loaded: "Loaded",
} as const;

export type LoadingState = typeof LoadingStates[keyof typeof LoadingStates];

export interface ChapterStore {
  chapter_id: string | null,
  loading_state: LoadingState,
  markdown: string | null,
  update_from_url: () => void,
}

export const useChapterStore: UseBoundStore<StoreApi<ChapterStore>> = create(immer((set) => ({
  chapter_id: null,
  loading_state: LoadingStates.Loading,
  markdown: null,
  update_from_url: () => {
    console.time("Load markdown");
    let url = new URL(window.location.href);
    let chapter_id = url.searchParams.get("chapter");
    console.log(chapter_id);
    set(state => {
      state.chapter_id = chapter_id
    });
    let timeout = setTimeout(() => {
      set((state: ChapterStore) => {
        if (state.chapter_id !== chapter_id) {
          return;
        }
        state.loading_state = LoadingStates.Loading;
        state.markdown = null;
      });
    }, 200);
    (async function () {
      if (!chapter_id) {
        return;
      }
      let chapter_data = await fetch(`api/chapter/${encodeURIComponent(chapter_id)}.md`);
      clearTimeout(timeout);
      let markdown = await chapter_data.text();
      set((state: ChapterStore) => {
        if (state.chapter_id !== chapter_id) {
          return;
        }
        state.loading_state = LoadingStates.Loaded;
        state.markdown = markdown;
        console.timeEnd("Load markdown");
      });
    })();
  }
})));

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