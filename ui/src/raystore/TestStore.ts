import {createStore} from "./raystore.ts";

interface TestStoreState {
  count: number;
}

export const TestStore = createStore({
  initialState: {
    count: 42,
  },
  actions: {
    increment(state: TestStoreState) {
      state.count++;
    },
  },
});