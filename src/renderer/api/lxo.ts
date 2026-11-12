import { invoke } from '@tauri-apps/api/core';

// Same positional calling convention as the Electron preload bridge.
// The Rust backend performs validation and owns all database access.
let sessionToken: string | null = null;
export function setSessionToken(token: string | null){ sessionToken = token; }
export const lxo = {
  invoke: (channel: string, ...args: unknown[]): Promise<any> =>
    invoke('lxo_call', { channel, args, session: sessionToken }),
  onWeight: (_callback: (weight: unknown) => void): (() => void) => {
    // No global TCP push listener in V2.6; explicit scale:capture is supported.
    return () => {};
  }
};
