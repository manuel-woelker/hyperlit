import * as React from "react";
import {produce} from "immer";

interface Raystore<STATE, ACTIONS extends ActionDefinitions<STATE>> {
  state: STATE,
  useState: () => STATE,
  update: (fn: (state: STATE) => void) => void,
  dispatch: ActionDispatchers<STATE, ACTIONS>,
  trigger: ActionTriggerMakers<STATE, ACTIONS>,
}

type ActionDefinition<STATE, PARAMETERS extends any[]> = (state: STATE, ...parameters: PARAMETERS) => void;
type ActionDefinitions<STATE> = {
  [key: string]: ActionDefinition<STATE, any>
};

type ActionDispatcher<PARAMETERS extends any[]> = (...parameters: PARAMETERS) => void;
type ActionDispatchers<STATE, ACTIONS extends ActionDefinitions<STATE>> = {
  [K in keyof ACTIONS]: ActionDispatcher<TailArguments<ACTIONS[K]>>
};

type ActionTriggerMaker<PARAMETERS extends any[]> = (...parameters: PARAMETERS) => () => void;
type ActionTriggerMakers<STATE, ACTIONS extends ActionDefinitions<STATE>> = {
  [K in keyof ACTIONS]: ActionTriggerMaker<TailArguments<ACTIONS[K]>>
};


//export type Stoar<STATE, ACTIONS> = StoarBase<STATE, ACTIONS>;

export function createStore<STATE, ACTIONS extends ActionDefinitions<STATE> = {}>(init: {
  initialState: STATE,
  actions?: ACTIONS
}): Raystore<STATE, ACTIONS> {
  let state = init.initialState;
  let subscribers: (() => void)[] = []

  function subscribe(callback: () => void) {
    console.log("Subscribing...")
    subscribers.push(callback);
    return () => {
      console.log("Unsubscribing...")
      subscribers = subscribers.filter((sub) => sub !== callback);
    }
  }

  function update(fn: (state: STATE) => void): void {
    state = produce(state, fn);
    subscribers.forEach((sub) => sub());
  }

  function getSnapshot(): STATE {
    return state;
  }

  function createActionDispatcher<PARAMETERS extends any[]>(actionDefinition: ActionDefinition<STATE, PARAMETERS>): ActionDispatcher<PARAMETERS> {
    return (...parameters: PARAMETERS) => {
      update((draft: STATE) => actionDefinition(draft, ...parameters));
    }
  }

  function createActionTriggerMaker<PARAMETERS extends any[]>(actionDefinition: ActionDefinition<STATE, PARAMETERS>): ActionTriggerMaker<PARAMETERS> {
    return (...parameters: PARAMETERS) => {
      return () => {
        update((draft: STATE) => actionDefinition(draft, ...parameters));
      }
    }
  }


  let actionDispatchers: ActionDispatchers<STATE, ACTIONS> = {} as ActionDispatchers<STATE, ACTIONS>;
  let actionTriggerMakers: ActionTriggerMakers<STATE, ACTIONS> = {} as ActionTriggerMakers<STATE, ACTIONS>;
  let actions = init.actions;
  if (actions) {
    for (let actionName in actions) {
      actionDispatchers[actionName] = createActionDispatcher(actions[actionName]);
      actionTriggerMakers[actionName] = createActionTriggerMaker(actions[actionName]);
    }
  }

  return {
    state,
    useState: function useState() {
      return React.useSyncExternalStore(subscribe, getSnapshot)
    },
    update,
    dispatch: actionDispatchers,
    trigger: actionTriggerMakers,
  }
}

type TailArguments<F extends (...args: any) => any> =
    F extends (first: any, ...rest: infer R) => any ? R : never;