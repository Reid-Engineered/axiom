import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { ProgressBar } from "../../components/primitives/ProgressBar";

describe("ProgressBar component", () => {
  it("renders with correct accessibility attributes", () => {
    render(<ProgressBar value={40} max={100} />);
    const bar = screen.getByRole("progressbar");
    expect(bar).toHaveAttribute("aria-valuenow", "40");
    expect(bar).toHaveAttribute("aria-valuemax", "100");
  });

  it("calculates fill percentage style correctly", () => {
    const { container } = render(<ProgressBar value={50} max={200} />);
    const fill = container.querySelector('[style*="width"]');
    expect(fill).toHaveStyle({ width: "25%" });
  });

  it("clamps values between 0 and 100 percent", () => {
    const { container } = render(<ProgressBar value={150} max={100} />);
    const fill = container.querySelector('[style*="width"]');
    expect(fill).toHaveStyle({ width: "100%" });
  });
});
