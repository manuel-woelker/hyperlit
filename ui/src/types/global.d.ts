interface Window {
  __REDUX_DEVTOOLS_EXTENSION__?: {
    connect: (options: { name?: string; instanceId?: string }) => ReduxDevTools;
  };
}

interface ReduxDevTools {
  init: (state: any) => void;
  send: (action: any, state: any) => void;
  subscribe: (listener: (message: any) => void) => () => void;
  unsubscribe: () => void;
}