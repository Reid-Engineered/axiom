import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { DiagnosticDot } from "../../components/badges/DiagnosticDot";

describe("DiagnosticDot component", () => {
  it("renders mistake dot with tooltip title", () => {
    render(<DiagnosticDot type="mistake" tooltip="Chose u backwards" />);
    const dot = screen.getByTitle("Chose u backwards");
    expect(dot).toBeInTheDocument();
    expect(dot.className).toMatch(/typeMistake/);
  });

  it("renders positive dot", () => {
    const { container } = render(<DiagnosticDot type="positive" size="lg" />);
    const dot = container.firstChild as HTMLElement;
    expect(dot.className).toMatch(/typePositive/);
    expect(dot.className).toMatch(/sizeLg/);
  });
});
