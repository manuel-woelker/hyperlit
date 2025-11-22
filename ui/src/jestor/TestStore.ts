import {createStore} from "./jestor.ts";

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
  derivedState: {
    doubleCount(state: TestStoreState) {
      if (state.flag) {
        return state.count * 2;
      } else {
        return state.count;
      }
    },
  },
});