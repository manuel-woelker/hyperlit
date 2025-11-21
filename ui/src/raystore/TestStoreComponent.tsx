import {TestStore} from "./TestStore.ts";

export function TestStoreComponent() {
  let state = TestStore.useState();
  console.log("render");
  return <div>
    Counter: {state.count}
    <button onClick={() => TestStore.update((state) => {
      state.count++
    })}>+
    </button>
    <button onClick={() => TestStore.dispatch.increment()}>INC</button>
    <button onClick={TestStore.trigger.increment()}>PLUS</button>
    <button onClick={TestStore.trigger.incrementBy(3)}>PLUS 3</button>
  </div>
}