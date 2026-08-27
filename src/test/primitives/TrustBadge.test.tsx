import { render, screen } from "@testing-library/react";
import { describe, it, expect } from "vitest";
import { TrustBadge } from "../../components/badges/TrustBadge";

describe("TrustBadge component", () => {
  it("renders verified badge with default label", () => {
    render(<TrustBadge type="verified" />);
    expect(screen.getByText("Axiom Verified")).toBeInTheDocument();
  });

  it("renders community badge with custom detail", () => {
    render(<TrustBadge type="community" detail="4.8k learners" />);
    expect(screen.getByText("Community")).toBeInTheDocument();
    expect(screen.getByText("· 4.8k learners")).toBeInTheDocument();
  });

  it("renders experimental badge", () => {
    render(<TrustBadge type="experimental" />);
    expect(screen.getByText("Experimental")).toBeInTheDocument();
  });
});
