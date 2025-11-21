interface Window {
  __REDUX_DEVTOOLS_EXTENSION__?: {
    connect: (options: { name?: string; instanceId?: string }) => {
      init: (state: any) => void;
      send: (action: string, state: any) => void;
      subscribe: (listener: (message: any) => void) => () => void;
      unsubscribe: () => void;
    };
  };
}
