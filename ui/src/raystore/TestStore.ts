import {createStore} from "./raystore.ts";

interface TestStoreState {
  count: number;
}

export const TestStore = createStore({
  name: "Teststore",
  initialState: {
    count: 42,
  },
  actions: {
    increment(state: TestStoreState) {
      state.count++;
    },
    incrementBy(state: TestStoreState, howMuch: number) {
      state.count += howMuch;
    },
  },
});