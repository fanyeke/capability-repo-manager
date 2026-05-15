import { describe, it, expect, beforeEach } from "vitest";
import { get } from "svelte/store";
import {
  report,
  isLoading,
  error,
  clearReport,
  clearError,
  groupIssuesBySeverity,
  getScoreColor,
  getScoreLabel,
} from "$lib/stores/doctorStore";
import type { DoctorIssue } from "$lib/types";

describe("doctorStore", () => {
  beforeEach(() => {
    report.set(null);
    isLoading.set(false);
    error.set(null);
  });

  describe("groupIssuesBySeverity", () => {
    const makeIssues = (severities: string[]): DoctorIssue[] =>
      severities.map((s) => ({
        severity: s as "critical" | "warning" | "info",
        code: "TEST",
        message: "test",
        resource_ref: null,
        recommendation: null,
      }));

    it("groups critical issues", () => {
      const issues = makeIssues(["critical", "warning", "critical"]);
      const groups = groupIssuesBySeverity(issues);
      expect(groups.critical).toHaveLength(2);
    });

    it("groups warning issues", () => {
      const issues = makeIssues(["warning", "info", "warning", "critical"]);
      const groups = groupIssuesBySeverity(issues);
      expect(groups.warning).toHaveLength(2);
    });

    it("groups info issues", () => {
      const issues = makeIssues(["info", "warning"]);
      const groups = groupIssuesBySeverity(issues);
      expect(groups.info).toHaveLength(1);
    });
  });

  describe("getScoreColor", () => {
    it("returns green for good scores", () => {
      expect(getScoreColor(100)).toBe("#22c55e");
      expect(getScoreColor(80)).toBe("#22c55e");
    });

    it("returns yellow for medium scores", () => {
      expect(getScoreColor(70)).toBe("#eab308");
      expect(getScoreColor(50)).toBe("#eab308");
    });

    it("returns red for poor scores", () => {
      expect(getScoreColor(49)).toBe("#ef4444");
      expect(getScoreColor(0)).toBe("#ef4444");
    });
  });

  describe("getScoreLabel", () => {
    it("returns Healthy for good scores", () => {
      expect(getScoreLabel(100)).toBe("Healthy");
      expect(getScoreLabel(80)).toBe("Healthy");
    });

    it("returns Needs Attention for medium scores", () => {
      expect(getScoreLabel(70)).toBe("Needs Attention");
      expect(getScoreLabel(50)).toBe("Needs Attention");
    });

    it("returns Critical for poor scores", () => {
      expect(getScoreLabel(49)).toBe("Critical");
      expect(getScoreLabel(0)).toBe("Critical");
    });
  });

  describe("clearReport", () => {
    it("resets report to null", () => {
      report.set({} as any);
      clearReport();
      expect(get(report)).toBeNull();
    });
  });

  describe("clearError", () => {
    it("resets error to null", () => {
      error.set("something went wrong");
      clearError();
      expect(get(error)).toBeNull();
    });
  });
});
