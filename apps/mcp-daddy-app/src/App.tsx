import { useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="min-h-full bg-[radial-gradient(60rem_35rem_at_10%_-10%,rgba(14,116,144,0.18),transparent_60%),radial-gradient(55rem_35rem_at_100%_0%,rgba(234,88,12,0.12),transparent_60%),linear-gradient(180deg,#ffffff,#fafafa)] text-zinc-900">
      <div className="mx-auto max-w-xl px-6 py-10">
        <header className="flex items-center justify-between gap-4">
          <div>
            <p className="text-xs font-medium tracking-wide text-zinc-600">
              MCP Daddy (desktop)
            </p>
            <h1 className="mt-1 text-2xl font-semibold tracking-tight">Tauri + React</h1>
          </div>

          <div className="flex items-center gap-2">
            <a href="https://tauri.app" target="_blank" rel="noreferrer">
              <img src="/tauri.svg" className="h-7 w-7" alt="Tauri" />
            </a>
            <a href="https://vite.dev" target="_blank" rel="noreferrer">
              <img src="/vite.svg" className="h-7 w-7" alt="Vite" />
            </a>
            <a href="https://react.dev" target="_blank" rel="noreferrer">
              <img src={reactLogo} className="h-7 w-7" alt="React" />
            </a>
          </div>
        </header>

        <section className="mt-8 rounded-2xl border border-zinc-200/70 bg-white/70 p-5 shadow-sm backdrop-blur">
          <p className="text-sm text-zinc-700">
            Tailwind is enabled. This is still the default Tauri template command, just styled.
          </p>

          <form
            className="mt-4 flex flex-col gap-3 sm:flex-row"
            onSubmit={(e) => {
              e.preventDefault();
              greet();
            }}
          >
            <input
              className="h-10 w-full rounded-lg border border-zinc-200 bg-white px-3 text-sm shadow-sm outline-none transition focus:border-zinc-400"
              onChange={(e) => setName(e.currentTarget.value)}
              placeholder="Enter a name..."
            />
            <Button type="submit">
              Greet
            </Button>
          </form>

          <p className="mt-3 min-h-6 text-sm text-zinc-700">{greetMsg}</p>
        </section>
      </div>
    </main>
  );
}

export default App;
