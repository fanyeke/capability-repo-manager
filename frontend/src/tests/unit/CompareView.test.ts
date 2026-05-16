import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/svelte";
import CompareView from "$lib/components/CompareView.svelte";
import type { CompareResult, CapabilityResource, DiffItem } from "$lib/types";

function makeResource(id: string, name: string, type: string): CapabilityResource {
  return {
    id,
    repo_id: "repo-1",
    pack_id: null,
    type: type as any,
    name,
    source_path: `/path/${name}`,
    scope: "project",
    tracked_by_git: true,
    content_hash: "abc",
    metadata_json: null,
    error_message: null,
  };
}

function makeResult(overrides: Partial<CompareResult> = {}): CompareResult {
  return {
    missing: [makeResource("m1", "missing-skill", "skill")],
    extra: [makeResource("e1", "extra-hook", "hook")],
    modified: [
      {
        name: "changed-rule",
        type: "rule",
        source_resource: makeResource("s1", "changed-rule", "rule"),
        target_resource: makeResource("t1", "changed-rule", "rule"),
      } as DiffItem,
    ],
    same: [makeResource("same1", "same-resource", "skill")],
    ...overrides,
  };
}

describe("CompareView", () => {
  const defaultProps = {
    result: null as CompareResult | null,
    selectedCategory: "missing" as const,
    onSelectCategory: vi.fn(),
  };

  it("shows empty state when no result", () => {
    const { container } = render(CompareView, { props: defaultProps });
    expect(container.textContent).toContain("暂无比较结果");
  });

  it("shows all category tabs", () => {
    const result = makeResult();
    const { container } = render(CompareView, {
      props: { ...defaultProps, result },
    });
    expect(container.textContent).toContain("缺失");
    expect(container.textContent).toContain("多余");
    expect(container.textContent).toContain("已修改");
    expect(container.textContent).toContain("相同");
  });

  it("renders missing items by default", () => {
    const result = makeResult();
    const { container } = render(CompareView, {
      props: { ...defaultProps, result },
    });
    expect(container.textContent).toContain("missing-skill");
  });

  it("renders extra items when category is extra", () => {
    const result = makeResult();
    const { container } = render(CompareView, {
      props: { ...defaultProps, result, selectedCategory: "extra" },
    });
    expect(container.textContent).toContain("extra-hook");
  });

  it("renders modified items when category is modified", () => {
    const result = makeResult();
    const { container } = render(CompareView, {
      props: { ...defaultProps, result, selectedCategory: "modified" },
    });
    expect(container.textContent).toContain("changed-rule");
  });

  it("renders same items when category is same", () => {
    const result = makeResult();
    const { container } = render(CompareView, {
      props: { ...defaultProps, result, selectedCategory: "same" },
    });
    expect(container.textContent).toContain("same-resource");
  });

  it("shows select hint when no resource selected", () => {
    const result = makeResult();
    const { container } = render(CompareView, {
      props: { ...defaultProps, result },
    });
    expect(container.textContent).toContain("选择一个项目以查看详情");
  });
});
