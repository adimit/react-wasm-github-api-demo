// A dependency graph that contains any wasm must all be imported
// asynchronously. This bootstrap file does the single async import, so
// that no one else needs to worry about it again.
import("./index-cra")
  .catch(e => console.error("Error importing `index-cra`:", e));

// Need to make this file a module
export {};
