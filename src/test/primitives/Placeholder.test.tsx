import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { Placeholder } from "../../components/primitives/Placeholder";

describe("Placeholder component", () => {
  it("renders label caption", () => {
    render(<Placeholder label="Visualizer Scene" />);
    expect(screen.getByText("Visualizer Scene")).toBeInTheDocument();
  });

  it("applies inline height and width styles if provided", () => {
    const { container } = render(
      <Placeholder label="3D Canvas" height={200} width="100%" />
    );
    const element = container.firstChild as HTMLElement;
    expect(element).toHaveStyle({ height: "200px", width: "100%" });
  });
});
