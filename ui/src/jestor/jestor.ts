import * as React from "react";
import {produce} from "immer";

/**
 * Core store interface that provides state management capabilities
 * @template STATE - The shape of the application state
 * @template ACTIONS - Type defining the available actions
 * @template STATE_DERIVATIONS - Type defining derived state values
 */
interface Jestor<
    STATE,
    ACTIONS extends ActionDefinitions<STATE>,
    STATE_DERIVATIONS extends StateDerivations<STATE>> {
  useState: () => CombinedState<STATE, STATE_DERIVATIONS>,
  getSnapshot: () => CombinedState<STATE, STATE_DERIVATIONS>,
  update: (label: string, fn: (state: STATE) => void) => void,
  dispatch: ActionDispatchers<STATE, ACTIONS>,
  trigger: ActionTriggerMakers<STATE, ACTIONS>,
  select: Selectors<CombinedState<STATE, STATE_DERIVATIONS>>,
  subscribe: (callback: () => void) => void;
}

/**
 * Combines base state with derived state
 * @template STATE - The base state type
 * @template STATE_DERIVATIONS - Type of derived state functions
 */
type CombinedState<STATE, STATE_DERIVATIONS extends StateDerivations<STATE>> =
    STATE
    & DerivedState<STATE, STATE_DERIVATIONS>;


/**
 * Defines an action function that can modify the state
 * @template STATE - The state type
 * @template PARAMETERS - Tuple type of action parameters
 */
type ActionDefinition<STATE, PARAMETERS extends any[]> = (state: STATE, ...parameters: PARAMETERS) => void;
/**
 * Object mapping action names to their implementations
 * @template STATE - The state type
 */
type ActionDefinitions<STATE> = {
  [key: string]: ActionDefinition<STATE, any>
};

/**
 * Function type for dispatching an action
 * @template PARAMETERS - Tuple type of action parameters
 */
type ActionDispatcher<PARAMETERS extends any[]> = (...parameters: PARAMETERS) => void;
/**
 * Maps action names to their corresponding dispatcher functions
 * @template STATE - The state type
 * @template ACTIONS - Type containing action definitions
 */
type ActionDispatchers<STATE, ACTIONS extends ActionDefinitions<STATE>> = {
  [K in keyof ACTIONS]: ActionDispatcher<TailArguments<ACTIONS[K]>>
};

/**
 * Creates an action trigger function for event handlers
 * @template PARAMETERS - Tuple type of action parameters
 */
type ActionTriggerMaker<PARAMETERS extends any[]> = (...parameters: PARAMETERS) => () => void;
/**
 * Maps action names to their corresponding trigger maker functions
 * @template STATE - The state type
 * @template ACTIONS - Type containing action definitions
 */
type ActionTriggerMakers<STATE, ACTIONS extends ActionDefinitions<STATE>> = {
  [K in keyof ACTIONS]: ActionTriggerMaker<TailArguments<ACTIONS[K]>>
};

/**
 * Function that selects a value from the state
 * @template T - Type of the selected value
 */
type Selector<T> = () => T;
/**
 * Maps state properties to their selector functions
 * @template STATE - The state type
 */
type Selectors<STATE> = {
  [K in keyof STATE]: Selector<STATE[K]>
}

type ReturnTypeOfFunction<F extends (...args: any[]) => any> = F extends (...args: any[]) => infer R ? R : never;
/**
 * Function that derives a value from the state
 * @template STATE - The state type
 * @template R - The type of the derived value
 */
type StateDerivation<STATE, R> = (state: STATE) => R;
/**
 * Maps derived state property names to their derivation functions
 * @template STATE - The state type
 */
type StateDerivations<STATE> = {
  [key: string]: StateDerivation<STATE, any>
};


/**
 * Type representing the derived state object
 * @template STATE - The state type
 * @template STATE_DERIVATIONS - Type containing derivation functions
 */
type DerivedState<STATE, STATE_DERIVATIONS extends StateDerivations<STATE>> = {
  [K in keyof STATE_DERIVATIONS]: ReturnTypeOfFunction<STATE_DERIVATIONS[K]>
}


/**
 * Creates a new state store with the given configuration
 * @template STATE - The shape of the application state
 * @template ACTIONS - Type defining the available actions
 * @template STATE_DERIVATIONS - Type defining derived state values
 * @param init - Configuration object for the store
 * @param init.name - Name of the store (used for devtools)
 * @param init.initialState - The initial state of the store
 * @param [init.actions] - Optional actions that can modify the state
 * @param [init.derivedState] - Optional derived state functions
 * @returns A configured store instance
 */
export function createStore<STATE, ACTIONS extends ActionDefinitions<STATE> = {}, STATE_DERIVATIONS extends StateDerivations<STATE> = {}>(init: {
  name: string,
  initialState: STATE,
  derivedState?: STATE_DERIVATIONS,
  actions?: ACTIONS,
}): Jestor<STATE, ACTIONS, STATE_DERIVATIONS> {
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
    subscribe,
    dispatch: actionDispatchers,
    trigger: actionTriggerMakers,
    select: selectors,
  }
}

/**
 * Extracts the parameter types of a function, excluding the first parameter
 * @template F - Function type to extract parameters from
 */
type TailArguments<F extends (...args: any) => any> =
    F extends (first: any, ...rest: infer R) => any ? R : never;