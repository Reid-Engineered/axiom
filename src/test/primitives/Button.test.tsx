import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { Button } from "../../components/primitives/Button";

describe("Button component", () => {
  it("renders children text correctly", () => {
    render(<Button>Click me</Button>);
    expect(screen.getByRole("button", { name: /click me/i })).toBeInTheDocument();
  });

  it("applies primary variant by default", () => {
    const { container } = render(<Button>Action</Button>);
    const button = container.querySelector("button");
    expect(button?.className).toMatch(/variantPrimary/);
  });

  it("applies variant and size classes", () => {
    const { container } = render(
      <Button variant="dark" size="lg">
        Dark Large
      </Button>
    );
    const button = container.querySelector("button");
    expect(button?.className).toMatch(/variantDark/);
    expect(button?.className).toMatch(/sizeLg/);
  });

  it("handles onClick events", () => {
    const handleClick = vi.fn();
    render(<Button onClick={handleClick}>Click</Button>);
    fireEvent.click(screen.getByRole("button"));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it("respects disabled prop", () => {
    const handleClick = vi.fn();
    render(
      <Button disabled onClick={handleClick}>
        Disabled
      </Button>
    );
    const button = screen.getByRole("button");
    expect(button).toBeDisabled();
    fireEvent.click(button);
    expect(handleClick).not.toHaveBeenCalled();
  });
});
