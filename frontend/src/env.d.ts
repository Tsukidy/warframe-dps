/// <reference types="vite/client" />

/** True when the app runs with MSW mocks (`vite --mode mock`); constant-folded, so production builds drop them. */
declare const __WFD_MOCKS__: boolean;
