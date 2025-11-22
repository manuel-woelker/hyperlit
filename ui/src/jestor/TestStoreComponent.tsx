import {TestStore} from "./TestStore.ts";
import * as React from "react";

export function TestStoreComponent() {
  console.log("render");
  return <div>
    <div>
      Counter: {TestStore.select.count()}
      <button onClick={() => TestStore.update((state) => {
        state.count++
      })}>+
      </button>
      <button onClick={() => TestStore.dispatch.increment()}>INC</button>
      <button onClick={TestStore.trigger.increment()}>PLUS</button>
      <button onClick={TestStore.trigger.incrementBy(3)}>PLUS 3</button>
      {TestStore.select.count()}
    </div>
    <ToggleComponent/>
    <div>
      Double Count: {TestStore.select.doubleCount()}
    </div>
  </div>
}

export const ToggleComponent = React.memo(function ToggleComponent() {
  console.log("render toggle");
  return <div>
    Flag: {TestStore.select.flag() ? "ON" : "OFF"}
    <button onClick={() => TestStore.dispatch.toggleFlag()}>TOGGLE</button>
    <button onClick={TestStore.trigger.toggleFlag()}>TOGGLE</button>
  </div>;
})