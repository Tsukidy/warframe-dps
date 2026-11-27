// The MSW mocks answer every route from the mini fixture with the generated types' shapes.
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { mockServer } from "./node";
import { api } from "../api/client";
import { newDocument } from "../state/defaults";
import { buildMcRequest } from "../api/seed";

beforeAll(() => mockServer.listen({ onUnhandledFrame: "error" }));
afterAll(() => mockServer.close());

describe("mock API", () => {
  it("serves meta, catalogue and schema from the mini fixture", async () => {
    const meta = await api.meta();
    expect(meta.app_id).toBe("warframe-dps");
    const cat = await api.catalog();
    expect(cat.data_version).toBe(meta.data_version);
    expect(cat.weapons.find((w) => w.key === 1204)?.name).toBe("Braton Prime");
    expect(cat.enemies.find((e) => e.key === 9001)?.name).toBe("Lancer");
    const schema = await api.schema();
    expect(schema.toggles.length).toBeGreaterThan(0);
    expect(schema.slot_layouts.primary?.slots).toContain("gen0");
  });

  it("round-trips a mock share code and answers calc with the code", async () => {
    const doc = newDocument(1204, 9001);
    const { code } = await api.encode(doc);
    const back = await api.decode(code);
    expect(back.input).toEqual(doc);
    const report = await api.calc(buildMcRequest(doc));
    expect(report.code).toBe(code);
    expect(report.attacks.length).toBeGreaterThan(0);
  });

  it("returns ErrorResponse bodies as ApiError messages", async () => {
    await expect(api.weapon(1)).rejects.toMatchObject({ status: 404, msg: { key: "api.not_found" } });
    await expect(api.decode("nonsense")).rejects.toMatchObject({ status: 422, msg: { key: "import.unrecognized" } });
  });
});
