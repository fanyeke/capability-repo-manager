import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import MigrationPlan from "$lib/components/MigrationPlan.svelte";
import type { MigrationPlan as MigrationPlanType, MigrationPlanItem, MigrationConflict } from "$lib/types";

function makePlan(overrides: Partial<MigrationPlanType> = {}): MigrationPlanType {
  const items: MigrationPlanItem[] = [
    {
      resource_id: "skill-1",
      action: "add",
      source_path: "/source/skill-a",
      target_path: "/target/skill-a",
      status: "pending",
    },
    {
      resource_id: "rule-1",
      action: "overwrite",
      source_path: "/source/rule-a",
      target_path: "/target/rule-a",
      status: "pending",
    },
  ];
  const conflicts: MigrationConflict[] = [
    {
      resource_name: "my-hook",
      resource_type: "hook",
      reason: "same_name",
      recommended_actions: ["skip", "rename"],
    },
  ];
  return {
    plan_id: "plan-1",
    source: { type: "pack", id: "pack-1", name: "test-pack" },
    target: { id: "repo-1", name: "target-repo" },
    items,
    conflicts,
    missing_deps: [],
    ...overrides,
  };
}

describe("MigrationPlan", () => {
  const defaultProps = {
    plan: null as MigrationPlanType | null,
    strategies: [] as { resource_id: string; action: string }[],
    onSetStrategy: vi.fn(),
  };

  it("shows empty state when no plan", () => {
    const { container } = render(MigrationPlan, { props: defaultProps });
    expect(container.textContent).toContain("No migration plan");
  });

  it("shows plan summary with source and target", () => {
    const plan = makePlan();
    const { container } = render(MigrationPlan, {
      props: { ...defaultProps, plan },
    });
    expect(container.textContent).toContain("test-pack");
    expect(container.textContent).toContain("target-repo");
  });

  it("shows item count in summary", () => {
    const plan = makePlan();
    const { container } = render(MigrationPlan, {
      props: { ...defaultProps, plan },
    });
    expect(container.textContent).toContain("2");
  });

  it("shows conflicts section when conflicts exist", () => {
    const plan = makePlan();
    const { container } = render(MigrationPlan, {
      props: { ...defaultProps, plan },
    });
    expect(container.textContent).toContain("Conflicts");
    expect(container.textContent).toContain("my-hook");
  });

  it("shows conflict reason text", () => {
    const plan = makePlan();
    const { container } = render(MigrationPlan, {
      props: { ...defaultProps, plan },
    });
    expect(container.textContent).toContain("same_name");
  });

  it("renders table with resource and action columns", () => {
    const plan = makePlan();
    const { container } = render(MigrationPlan, {
      props: { ...defaultProps, plan },
    });
    expect(container.textContent).toContain("Resource");
    expect(container.textContent).toContain("Strategy");
  });

  it("renders strategy select for overwrite items", () => {
    const plan = makePlan();
    const { container } = render(MigrationPlan, {
      props: { ...defaultProps, plan },
    });
    const selects = container.querySelectorAll("select");
    expect(selects.length).toBeGreaterThan(0);
  });

  it("shows conflict count as 0 when no conflicts", () => {
    const plan = makePlan({ conflicts: [] });
    const { container } = render(MigrationPlan, {
      props: { ...defaultProps, plan },
    });
    expect(container.textContent).toContain("0");
  });

  it("shows action badges for items", () => {
    const plan = makePlan();
    const { container } = render(MigrationPlan, {
      props: { ...defaultProps, plan },
    });
    expect(container.textContent).toContain("add");
    expect(container.textContent).toContain("overwrite");
  });
});
