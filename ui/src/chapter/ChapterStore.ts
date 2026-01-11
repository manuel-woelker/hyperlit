import {createStore} from "../jestor/jestor.ts";

const LoadingStates = {
  Loading: "Loading",
  Loaded: "Loaded",
} as const;

export type LoadingState = typeof LoadingStates[keyof typeof LoadingStates];

export interface ChapterState {
  chapter_id: string | null,
  loading_state: LoadingState,
  markdown: string | null,
  edit_url: string | null,
}

export interface ChapterJson {
  chapter_id: string,
  markdown: string,
  edit_url: string | null,
}

export interface Document {
  id: string,
  title: string,
  markdown: string,
  edit_url: string | null,
}

export const chapterStore = createStore({
  name: "Chapter",
  initialState: {
    chapter_id: null,
    loading_state: LoadingStates.Loading,
    markdown: null,
    edit_url: null,
  },
  actions: {
    update_from_url: (state: ChapterState) => {
      console.time("Load markdown");
      let url = new URL(window.location.href);
      let chapter_id = url.searchParams.get("chapter");
      state.chapter_id = chapter_id
      if (!chapter_id) {
        return;
      }
      let timeout = setTimeout(() => {
        chapterStore.update("set loading state", (state: ChapterState) => {
          if (state.chapter_id !== chapter_id) {
            return;
          }
          state.loading_state = LoadingStates.Loading;
          state.markdown = null;
        });
      }, 200);
      (async function () {
        let chapter_data = await fetch(`api/document/${encodeURIComponent(chapter_id)}.json`);
        let document = await chapter_data.json() as Document;
        clearTimeout(timeout);
        chapterStore.update("Book loaded", (state: ChapterState) => {
          if (state.chapter_id !== chapter_id) {
            return;
          }
          state.loading_state = LoadingStates.Loaded;
          state.markdown = document.markdown;
          state.edit_url = document.edit_url;
          console.timeEnd("Load markdown");
        });
      })();
    }
  }
});

chapterStore.dispatch.update_from_url();

document.addEventListener('click', function (event) {
  const link = (event.target as Element)?.closest('a');
  if (!link) return;

  // Ignore modified clicks
  if (event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
  if (link.target === "_blank") return;

  const url = new URL(link.href);

  // Only handle internal navigation
  if (url.origin !== window.location.origin) return;

  event.preventDefault();
  window.history.pushState({}, '', link.href);

  // Update page content
  console.log('Navigated to:', link.href);
  chapterStore.dispatch.update_from_url();
});

// Handle back/forward navigation too
window.addEventListener('popstate', () => {
  console.log('User navigated with Back/Forward. Page:', window.location.href);
  chapterStore.dispatch.update_from_url();
});