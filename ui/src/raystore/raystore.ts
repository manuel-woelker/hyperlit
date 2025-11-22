import * as React from "react";
import {produce} from "immer";

interface Raystore<
    STATE,
    ACTIONS extends ActionDefinitions<STATE>,
    STATE_DERIVATIONS extends StateDerivations<STATE>> {
  useState: () => CombinedState<STATE, STATE_DERIVATIONS>,
  getSnapshot: () => CombinedState<STATE, STATE_DERIVATIONS>,
  update: (label: string, fn: (state: STATE) => void) => void,
  dispatch: ActionDispatchers<STATE, ACTIONS>,
  trigger: ActionTriggerMakers<STATE, ACTIONS>,
  select: Selectors<CombinedState<STATE, STATE_DERIVATIONS>>,
}

type CombinedState<STATE, STATE_DERIVATIONS extends StateDerivations<STATE>> =
    STATE
    & DerivedState<STATE, STATE_DERIVATIONS>;


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

type ReturnTypeOfFunction<F extends (...args: any[]) => any> = F extends (...args: any[]) => infer R ? R : never;
type StateDerivation<STATE, R> = (state: STATE) => R;
type StateDerivations<STATE> = {
  [key: string]: StateDerivation<STATE, any>
};


type DerivedState<STATE, STATE_DERIVATIONS extends StateDerivations<STATE>> = {
  [K in keyof STATE_DERIVATIONS]: ReturnTypeOfFunction<STATE_DERIVATIONS[K]>
}


export function createStore<STATE, ACTIONS extends ActionDefinitions<STATE> = {}, STATE_DERIVATIONS extends StateDerivations<STATE> = {}>(init: {
  name: string,
  initialState: STATE,
  derivedState?: STATE_DERIVATIONS,
  actions?: ACTIONS,
}): Raystore<STATE, ACTIONS, STATE_DERIVATIONS> {
  type FullState = CombinedState<STATE, STATE_DERIVATIONS>;
  let devTools: ReduxDevTools | null = null;
  if (window.__REDUX_DEVTOOLS_EXTENSION__) {
    devTools = window.__REDUX_DEVTOOLS_EXTENSION__.connect({
      name: init.name,
      instanceId: init.name,
    });
    devTools.init(init.initialState);
  }
  let baseState = init.initialState;
  let subscribers: (() => void)[] = []

  function computeDerivedState(): DerivedState<STATE, STATE_DERIVATIONS> {
    let derivedState: DerivedState<STATE, STATE_DERIVATIONS> = {} as DerivedState<STATE, STATE_DERIVATIONS>;
    if (init.derivedState) {
      for (let key in init.derivedState) {
        derivedState[key] = init.derivedState[key](baseState);
      }
    }
    return derivedState;
  }

  let derivedState: DerivedState<STATE, STATE_DERIVATIONS> = {} as DerivedState<STATE, STATE_DERIVATIONS>;

  let combinedState: FullState = {} as FullState;

  function updateCombinedState() {
    derivedState = computeDerivedState();
    combinedState = {...baseState, ...derivedState};
  }

  updateCombinedState();
  devTools?.send({type: "@@DERIVED_STATE"}, combinedState);

  function subscribe(callback: () => void) {
    subscribers.push(callback);
    return () => {
      subscribers = subscribers.filter((sub) => sub !== callback);
    }
  }

  function update(label: string, fn: (state: STATE) => void): void {
    updateInternal(fn, {type: label});
  }

  function updateInternal(fn: (state: STATE) => void, devInfo: any): void {
    baseState = produce(baseState, fn);
    updateCombinedState();
    subscribers.forEach((sub) => sub());
    devTools?.send(devInfo, combinedState);
  }


  function applyAction<PARAMETERS extends any[]>(name: string, actionDefinition: ActionDefinition<STATE, PARAMETERS>, parameters: PARAMETERS): void {
    updateInternal((draft: STATE) => actionDefinition(draft, ...parameters), {type: name, parameters});
  }

  function getSnapshot(): FullState {
    return combinedState;
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

  let selectors: Selectors<FullState> = {} as Selectors<FullState>;
  for (let key in baseState) {
    selectors[key] = () => {
      return React.useSyncExternalStore(subscribe, () => combinedState[key]);
    };
  }
  if (init.derivedState) {
    for (let key in init.derivedState) {
      selectors[key] = () => {
        return React.useSyncExternalStore(subscribe, () => combinedState[key]);
      };
    }
  }


  return {
    useState,
    getSnapshot,
    update,
    dispatch: actionDispatchers,
    trigger: actionTriggerMakers,
    select: selectors,
  }
}

type TailArguments<F extends (...args: any) => any> =
    F extends (first: any, ...rest: infer R) => any ? R : never;