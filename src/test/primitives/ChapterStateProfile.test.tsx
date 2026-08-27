import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { ChapterStateProfile } from "../../components/mastery/ChapterStateProfile";

describe("ChapterStateProfile component", () => {
  it("renders total concept count and mastery rings", () => {
    render(
      <ChapterStateProfile
        counts={{ Mastered: 2, Strong: 1, Developing: 1 }}
      />
    );
    expect(screen.getByText("4 concepts")).toBeInTheDocument();
  });

  it("handles singular concept count text", () => {
    render(<ChapterStateProfile counts={{ Mastered: 1 }} />);
    expect(screen.getByText("1 concept")).toBeInTheDocument();
  });
});
