import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { Toggle } from "../../components/primitives/Toggle";

describe("Toggle component", () => {
  it("renders with aria-checked reflecting state", () => {
    render(<Toggle checked={true} onChange={() => {}} label="Tutor active" />);
    const switchEl = screen.getByRole("switch");
    expect(switchEl).toHaveAttribute("aria-checked", "true");
    expect(screen.getByText("Tutor active")).toBeInTheDocument();
  });

  it("triggers onChange when clicked", () => {
    const handleChange = vi.fn();
    render(<Toggle checked={false} onChange={handleChange} label="Toggle me" />);
    fireEvent.click(screen.getByText("Toggle me"));
    expect(handleChange).toHaveBeenCalledWith(true);
  });

  it("supports keyboard interaction (Space/Enter)", () => {
    const handleChange = vi.fn();
    render(<Toggle checked={false} onChange={handleChange} label="Key toggle" />);
    const switchEl = screen.getByRole("switch");
    fireEvent.keyDown(switchEl, { key: "Enter" });
    expect(handleChange).toHaveBeenCalledWith(true);
  });

  it("does not trigger onChange when disabled", () => {
    const handleChange = vi.fn();
    render(<Toggle checked={false} onChange={handleChange} disabled label="Disabled toggle" />);
    fireEvent.click(screen.getByText("Disabled toggle"));
    expect(handleChange).not.toHaveBeenCalled();
  });
});
