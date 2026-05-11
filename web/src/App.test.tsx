import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { App } from "./App";

describe("App", () => {
  it("renders the shell with a stable test id", () => {
    render(<App />);
    expect(screen.getByTestId("app-root")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: /furnigen/i })).toBeInTheDocument();
  });

  it("shows a WASM status line while loading or after resolution", () => {
    render(<App />);
    expect(screen.getByTestId("wasm-status")).toHaveTextContent(
      /Loading WASM|WASM failed|WardrobeSpec v1 \+ preview mesh/
    );
  });
});
