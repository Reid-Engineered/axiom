import { render, screen, fireEvent } from "@testing-library/react";
import { describe, it, expect, vi } from "vitest";
import { Chip } from "../../components/primitives/Chip";

describe("Chip component", () => {
  it("renders label text", () => {
    render(<Chip label="Calculus II" />);
    expect(screen.getByText("Calculus II")).toBeInTheDocument();
  });

  it("does not render remove button by default", () => {
    render(<Chip label="Integration" />);
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  it("renders remove button when removable is true and fires onRemove", () => {
    const handleRemove = vi.fn();
    render(<Chip label="Deadline · Dec 12" removable onRemove={handleRemove} />);
    const removeBtn = screen.getByRole("button", { name: /remove deadline · dec 12/i });
    expect(removeBtn).toBeInTheDocument();
    fireEvent.click(removeBtn);
    expect(handleRemove).toHaveBeenCalledTimes(1);
  });

  it("applies variant classes", () => {
    const { container } = render(<Chip label="Accent" variant="accent" />);
    const chip = container.querySelector("span");
    expect(chip?.className).toMatch(/variantAccent/);
  });
});
