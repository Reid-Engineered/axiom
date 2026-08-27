import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { Mastery } from "../../components/mastery/Mastery";
import type { MasteryState } from "../../types";

describe("Mastery component", () => {
  const states: MasteryState[] = [
    "New",
    "Developing",
    "Familiar",
    "Strong",
    "Mastered",
  ];

  states.forEach((st) => {
    it(`renders ${st} state with mandatory reading-distance label`, () => {
      render(<Mastery state={st} />);
      expect(screen.getByText(st)).toBeInTheDocument();
    });
  });

  it("can hide text label if showLabel is false", () => {
    render(<Mastery state="Strong" showLabel={false} />);
    expect(screen.queryByText("Strong")).not.toBeInTheDocument();
    expect(screen.getByTitle("Mastery: Strong")).toBeInTheDocument();
  });

  it("applies size classes correctly", () => {
    const { container } = render(<Mastery state="Familiar" size="sm" />);
    const ring = container.querySelector('[class*="ringSm"]');
    expect(ring).toBeInTheDocument();
  });
});
