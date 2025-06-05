/// <reference types="vite/client" />

// Example: src/env.d.ts or src/shims-css.d.ts
declare module '*.css' {
  const content: Record<string, string>;
  export default content;
}
