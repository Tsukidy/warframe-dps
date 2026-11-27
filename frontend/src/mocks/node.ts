// MSW for Vitest (component tests that need the API).
import { setupServer } from "msw/node";
import { handlers } from "./handlers";

export const mockServer = setupServer(...handlers);
