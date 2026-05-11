export function App() {
  return (
    <main
      className="flex min-h-screen flex-col items-center justify-center gap-4 p-8"
      data-testid="app-root"
    >
      <h1 className="text-2xl font-semibold tracking-tight">FurniGen</h1>
      <p className="max-w-md text-center text-slate-400">
        Parametric wardrobe preview (Rust core + WASM + web). Units in the spec are
        millimeters.
      </p>
    </main>
  );
}
