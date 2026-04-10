declare module 'bun:test' {
  export const describe: (name: string, fn: () => void) => void;
  export const test: (name: string, fn: () => void | Promise<void>) => void;
  export const expect: (value: unknown) => {
    toBe: (expected: unknown) => void;
    toContain: (expected: string) => void;
    not: {
      toContain: (expected: string) => void;
    };
  };
}

declare module 'node:fs' {
  export function existsSync(path: string): boolean;
  export function readFileSync(path: string, encoding: string): string;
}

declare module 'node:path' {
  export function resolve(...paths: string[]): string;
}

interface ImportMeta {
  readonly dir: string;
}
