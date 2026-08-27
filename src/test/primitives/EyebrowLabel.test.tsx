import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { EyebrowLabel } from "../../components/primitives/EyebrowLabel";

describe("EyebrowLabel component", () => {
  it("renders text content in uppercase styling", () => {
    render(<EyebrowLabel>Continue</EyebrowLabel>);
    expect(screen.getByText("Continue")).toBeInTheDocument();
  });
});
