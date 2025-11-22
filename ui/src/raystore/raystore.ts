import * as React from "react";
import {produce} from "immer";

interface Raystore<STATE, ACTIONS extends ActionDefinitions<STATE>> {
  state: STATE,
  useState: () => STATE,
  update: (fn: (state: STATE) => void) => void,
  dispatch: ActionDispatchers<STATE, ACTIONS>,
  trigger: ActionTriggerMakers<STATE, ACTIONS>,
  select: Selectors<STATE>,
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

type Selector<T> = () => T;
type Selectors<STATE> = {
  [K in keyof STATE]: Selector<STATE[K]>
}


export function createStore<STATE, ACTIONS extends ActionDefinitions<STATE> = {}>(init: {
  name: string,
  initialState: STATE,
  actions?: ACTIONS
}): Raystore<STATE, ACTIONS> {
  let devTools: ReduxDevTools | null = null;
  if (window.__REDUX_DEVTOOLS_EXTENSION__) {
    devTools = window.__REDUX_DEVTOOLS_EXTENSION__.connect({
      name: init.name,
      instanceId: init.name,
    });
    devTools.init(init.initialState);
  }
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

  function applyAction<PARAMETERS extends any[]>(name: string, actionDefinition: ActionDefinition<STATE, PARAMETERS>, parameters: PARAMETERS): void {
    update((draft: STATE) => actionDefinition(draft, ...parameters));
    devTools?.send({parameters, type: name}, state);
  }

  function getSnapshot(): STATE {
    return state;
  }

  function createActionDispatcher<PARAMETERS extends any[]>(name: string, actionDefinition: ActionDefinition<STATE, PARAMETERS>): ActionDispatcher<PARAMETERS> {
    return (...parameters: PARAMETERS) => {
      applyAction(name, actionDefinition, parameters);
    }
  }

  function createActionTriggerMaker<PARAMETERS extends any[]>(name: string, actionDefinition: ActionDefinition<STATE, PARAMETERS>): ActionTriggerMaker<PARAMETERS> {
    return (...parameters: PARAMETERS) => {
      return () => {
        applyAction(name, actionDefinition, parameters);
      }
    }
  }


  let actionDispatchers: ActionDispatchers<STATE, ACTIONS> = {} as ActionDispatchers<STATE, ACTIONS>;
  let actionTriggerMakers: ActionTriggerMakers<STATE, ACTIONS> = {} as ActionTriggerMakers<STATE, ACTIONS>;
  let actions = init.actions;
  if (actions) {
    for (let actionName in actions) {
      actionDispatchers[actionName] = createActionDispatcher(actionName, actions[actionName]);
      actionTriggerMakers[actionName] = createActionTriggerMaker(actionName, actions[actionName]);
    }
  }

  function useState() {
    return React.useSyncExternalStore(subscribe, getSnapshot)
  }

  let selectors: Selectors<STATE> = {} as Selectors<STATE>;
  for (let key in state) {
    selectors[key] = () => {
      return React.useSyncExternalStore(subscribe, () => state[key]);
    };
  }

  return {
    state,
    useState,
    update,
    dispatch: actionDispatchers,
    trigger: actionTriggerMakers,
    select: selectors,
  }
}

type TailArguments<F extends (...args: any) => any> =
    F extends (first: any, ...rest: infer R) => any ? R : never;