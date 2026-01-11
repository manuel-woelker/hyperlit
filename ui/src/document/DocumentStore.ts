import {createStore} from "../jestor/jestor.ts";

const LoadingStates = {
  Loading: "Loading",
  Loaded: "Loaded",
} as const;

export type LoadingState = typeof LoadingStates[keyof typeof LoadingStates];

export interface DocumentState {
  document_id: string | null,
  loading_state: LoadingState,
  markdown: string | null,
  edit_url: string | null,
}

export interface Document {
  id: string,
  title: string,
  markdown: string,
  edit_url: string | null,
}

export const documentStore = createStore({
  name: "Document",
  initialState: {
    document_id: null,
    loading_state: LoadingStates.Loading,
    markdown: null,
    edit_url: null,
  },
  actions: {
    update_from_url: (state: DocumentState) => {
      console.time("Load markdown");
      let url = new URL(window.location.href);
      let document_id = url.searchParams.get("document");
      state.document_id = document_id
      if (!document_id) {
        return;
      }
      let timeout = setTimeout(() => {
        documentStore.update("set loading state", (state: DocumentState) => {
          if (state.document_id !== document_id) {
            return;
          }
          state.loading_state = LoadingStates.Loading;
          state.markdown = null;
        });
      }, 200);
      (async function () {
        let document_data = await fetch(`api/document/${encodeURIComponent(document_id)}.json`);
        let document = await document_data.json() as Document;
        clearTimeout(timeout);
        documentStore.update("Book loaded", (state: DocumentState) => {
          if (state.document_id !== document_id) {
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

documentStore.dispatch.update_from_url();

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
  documentStore.dispatch.update_from_url();
});

// Handle back/forward navigation too
window.addEventListener('popstate', () => {
  console.log('User navigated with Back/Forward. Page:', window.location.href);
  documentStore.dispatch.update_from_url();
});