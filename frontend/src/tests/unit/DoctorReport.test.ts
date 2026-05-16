import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/svelte";
import DoctorReport from "$lib/components/DoctorReport.svelte";
import type { DoctorReport as DoctorReportType, DoctorIssue } from "$lib/types";

function makeReport(score: number, issues: DoctorIssue[]): DoctorReportType {
  return {
    id: "report-1",
    repo_id: "repo-1",
    score,
    issues,
    created_at: "2026-01-01T00:00:00Z",
  };
}

describe("DoctorReport", () => {
  it("shows empty state when no report", () => {
    const { container } = render(DoctorReport, { props: { report: null } });
    expect(container.textContent).toContain("暂无健康报告");
  });

  it("shows score value", () => {
    const report = makeReport(85, []);
    const { container } = render(DoctorReport, { props: { report } });
    expect(container.textContent).toContain("85");
  });

  it("shows Healthy label for high score", () => {
    const report = makeReport(85, []);
    const { container } = render(DoctorReport, { props: { report } });
    expect(container.textContent).toContain("健康");
  });

  it("shows Needs Attention label for medium score", () => {
    const report = makeReport(60, []);
    const { container } = render(DoctorReport, { props: { report } });
    expect(container.textContent).toContain("需要注意");
  });

  it("shows Critical label for low score", () => {
    const report = makeReport(30, []);
    const { container } = render(DoctorReport, { props: { report } });
    expect(container.textContent).toContain("严重");
  });

  it("shows No Issues when report has empty issues", () => {
    const report = makeReport(100, []);
    const { container } = render(DoctorReport, { props: { report } });
    expect(container.textContent).toContain("未发现问题");
  });

  it("shows issue count per severity", () => {
    const issues: DoctorIssue[] = [
      { severity: "critical", code: "MISSING_HOOK", message: "Missing pre-commit hook", resource_ref: null, recommendation: null },
      { severity: "warning", code: "UNUSED_RULE", message: "Unused rule file", resource_ref: null, recommendation: null },
      { severity: "warning", code: "EMPTY_SKILL", message: "Empty skill", resource_ref: null, recommendation: null },
    ];
    const report = makeReport(70, issues);
    const { container } = render(DoctorReport, { props: { report } });
    expect(container.textContent).toContain("严重");
    expect(container.textContent).toContain("警告");
  });

  it("shows issue message", () => {
    const issues: DoctorIssue[] = [
      { severity: "critical", code: "MISSING_HOOK", message: "Pre-commit hook is missing", resource_ref: null, recommendation: null },
    ];
    const report = makeReport(80, issues);
    const { container } = render(DoctorReport, { props: { report } });
    expect(container.textContent).toContain("Pre-commit hook is missing");
  });

  it("shows recommendation when present", () => {
    const issues: DoctorIssue[] = [
      { severity: "warning", code: "DEPRECATED_SKILL", message: "Old skill format", resource_ref: null, recommendation: "Migrate to new format" },
    ];
    const report = makeReport(80, issues);
    const { container } = render(DoctorReport, { props: { report } });
    expect(container.textContent).toContain("Migrate to new format");
  });

  it("shows resource ref when present", () => {
    const issues: DoctorIssue[] = [
      { severity: "info", code: "INFO", message: "Info message", resource_ref: ".claude/skills/test", recommendation: null },
    ];
    const report = makeReport(95, issues);
    const { container } = render(DoctorReport, { props: { report } });
    expect(container.textContent).toContain(".claude/skills/test");
  });

  it("shows diagnosed date indicator", () => {
    const report = makeReport(90, []);
    const { container } = render(DoctorReport, { props: { report } });
    expect(container.textContent).toContain("诊断时间");
  });

  it("does not show critical group when no critical issues", () => {
    const issues: DoctorIssue[] = [
      { severity: "info", code: "INFO", message: "Info only", resource_ref: null, recommendation: null },
    ];
    const report = makeReport(99, issues);
    const { container } = render(DoctorReport, { props: { report } });
    expect(container.textContent).toContain("信息");
  });
});
