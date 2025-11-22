import {createStore} from "./raystore.ts";

interface TestStoreState {
  count: number;
  flag: boolean;
}

export const TestStore = createStore({
  name: "Teststore",
  initialState: {
    count: 42,
    flag: false,
  },
  actions: {
    increment(state: TestStoreState) {
      state.count++;
    },
    incrementBy(state: TestStoreState, howMuch: number) {
      state.count += howMuch;
    },
    toggleFlag(state: TestStoreState) {
      state.flag = !state.flag;
    },
  },
});