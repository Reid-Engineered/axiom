import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { OfflineChip } from "../../components/badges/OfflineChip";

describe("OfflineChip component", () => {
  it("renders Works offline status", () => {
    render(<OfflineChip status="Works offline" />);
    expect(screen.getByText("Works offline")).toBeInTheDocument();
  });

  it("normalizes short status literal names", () => {
    render(<OfflineChip status="required" />);
    expect(screen.getByText("Internet required")).toBeInTheDocument();
  });

  it("renders Online enhanced status", () => {
    render(<OfflineChip status="enhanced" />);
    expect(screen.getByText("Online enhanced")).toBeInTheDocument();
  });
});
