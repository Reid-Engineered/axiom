import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { SegmentedControl } from "../../components/primitives/SegmentedControl";

describe("SegmentedControl component", () => {
  const options = [
    { value: "explain", label: "Explain" },
    { value: "practice", label: "Practice" },
    { value: "reflect", label: "Reflect" },
  ];

  it("renders all options and highlights active option", () => {
    render(
      <SegmentedControl options={options} value="practice" onChange={() => {}} />
    );
    expect(screen.getByRole("tab", { name: "Explain" })).toHaveAttribute("aria-selected", "false");
    expect(screen.getByRole("tab", { name: "Practice" })).toHaveAttribute("aria-selected", "true");
    expect(screen.getByRole("tab", { name: "Reflect" })).toHaveAttribute("aria-selected", "false");
  });

  it("calls onChange when an unselected tab is clicked", () => {
    const handleChange = vi.fn();
    render(
      <SegmentedControl options={options} value="explain" onChange={handleChange} />
    );
    fireEvent.click(screen.getByRole("tab", { name: "Reflect" }));
    expect(handleChange).toHaveBeenCalledWith("reflect");
  });
});
