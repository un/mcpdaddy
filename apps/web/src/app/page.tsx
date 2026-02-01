import { Button } from '@/components/ui/button';

export default function Home() {
  return (
    <main className="relative isolate min-h-dvh overflow-hidden">
      <div className="pointer-events-none absolute inset-0 -z-10 bg-[radial-gradient(60rem_40rem_at_20%_0%,rgba(14,116,144,0.18),transparent_55%),radial-gradient(55rem_35rem_at_90%_10%,rgba(234,88,12,0.14),transparent_60%),linear-gradient(180deg,#fafafa,rgba(250,250,250,0.6))]" />

      <div className="mx-auto max-w-5xl px-6">
        <section className="flex min-h-dvh flex-col justify-center py-16 sm:py-24">
          <div className="inline-flex w-fit items-center gap-2 rounded-full border border-border bg-background/70 px-3 py-1 text-xs font-medium text-muted-foreground shadow-sm backdrop-blur">
            <span className="inline-block h-1.5 w-1.5 rounded-full bg-teal-600" />
            Local-first MCP proxy (alpha)
          </div>

          <h1 className="mt-6 text-balance text-4xl font-semibold tracking-tight sm:text-6xl">
            One MCP server.
            <span className="block text-muted-foreground">Many upstreams. Per-client control.</span>
          </h1>

          <p className="mt-6 max-w-2xl text-pretty text-lg leading-8 text-muted-foreground">
            MCP Daddy sits between your clients and upstream MCP servers, giving you a simple way to
            expose only the tools a client should see.
          </p>

          <div className="mt-10 flex flex-col gap-3 sm:flex-row">
            <Button asChild>
              <a href="#how-it-works">See how it works</a>
            </Button>
            <Button variant="outline" asChild>
              <a href="https://github.com/un/mcpdaddy" target="_blank" rel="noopener noreferrer">
                GitHub
              </a>
            </Button>
          </div>
        </section>

        <section id="how-it-works" className="scroll-mt-24 py-24">
          <h2 className="text-2xl font-semibold tracking-tight">How it works</h2>
          <p className="mt-3 max-w-3xl text-sm leading-7 text-muted-foreground">
            Think of MCP Daddy as a proxy: upstream servers on one side, your MCP clients on the
            other. For each client, you pick what upstreams (and therefore which tools) are visible.
          </p>

          <div className="mt-8 flex flex-col gap-4 sm:flex-row sm:items-stretch">
            <div className="flex-1 rounded-xl border border-border bg-background/70 p-5 shadow-sm backdrop-blur">
              <p className="text-xs font-medium text-muted-foreground">Upstreams</p>
              <p className="mt-2 text-base font-semibold">stdio MCP servers</p>
              <p className="mt-2 text-sm leading-6 text-muted-foreground">
                Each upstream runs as a local process. MCP Daddy launches them with the right
                command, args, and env.
              </p>
            </div>

            <div className="flex items-center justify-center py-1 text-muted-foreground sm:hidden">
              v
            </div>
            <div className="hidden items-center justify-center px-2 text-muted-foreground sm:flex">
              {'->'}
            </div>

            <div className="flex-1 rounded-xl border border-border bg-background/70 p-5 shadow-sm backdrop-blur">
              <p className="text-xs font-medium text-muted-foreground">MCP Daddy</p>
              <p className="mt-2 text-base font-semibold">Profiles + routing</p>
              <p className="mt-2 text-sm leading-6 text-muted-foreground">
                A single MCP endpoint that aggregates, filters, and routes tool calls based on a
                selected client profile.
              </p>
            </div>

            <div className="flex items-center justify-center py-1 text-muted-foreground sm:hidden">
              v
            </div>
            <div className="hidden items-center justify-center px-2 text-muted-foreground sm:flex">
              {'->'}
            </div>

            <div className="flex-1 rounded-xl border border-border bg-background/70 p-5 shadow-sm backdrop-blur">
              <p className="text-xs font-medium text-muted-foreground">Clients</p>
              <p className="mt-2 text-base font-semibold">Cursor, Claude Desktop, etc.</p>
              <p className="mt-2 text-sm leading-6 text-muted-foreground">
                Clients connect once. They see only what the profile allows, and call tools through
                MCP Daddy.
              </p>
            </div>
          </div>
        </section>

        <section id="security" className="scroll-mt-24 border-t border-border py-24">
          <h2 className="text-2xl font-semibold tracking-tight">Security &amp; privacy</h2>
          <p className="mt-3 max-w-3xl text-sm leading-7 text-muted-foreground">
            MCP Daddy is designed to be local-first. It does not make security promises it cannot
            keep, but it does make dangerous actions explicit.
          </p>

          <div className="mt-8 grid gap-4 sm:grid-cols-3">
            <div className="rounded-xl border border-border bg-background/70 p-5 shadow-sm backdrop-blur">
              <p className="text-base font-semibold">Local by default</p>
              <p className="mt-2 text-sm leading-6 text-muted-foreground">
                Runs on your machine and connects to upstreams as local processes unless you
                configure otherwise.
              </p>
            </div>
            <div className="rounded-xl border border-border bg-background/70 p-5 shadow-sm backdrop-blur">
              <p className="text-base font-semibold">Consent before commands</p>
              <p className="mt-2 text-sm leading-6 text-muted-foreground">
                Before executing a run-command request, the UI shows the exact command and args and
                requires explicit approval.
              </p>
            </div>
            <div className="rounded-xl border border-border bg-background/70 p-5 shadow-sm backdrop-blur">
              <p className="text-base font-semibold">Localhost safety</p>
              <p className="mt-2 text-sm leading-6 text-muted-foreground">
                When enabled, downstream endpoints are intended to bind to localhost and can enforce
                per-client tokens and origin checks.
              </p>
            </div>
          </div>
        </section>

        <section id="roadmap" className="scroll-mt-24 border-t border-border py-24">
          <h2 className="text-2xl font-semibold tracking-tight">Roadmap</h2>
          <div className="mt-8 grid gap-4 sm:grid-cols-2">
            <div className="rounded-xl border border-border bg-background/70 p-5 shadow-sm backdrop-blur">
              <p className="text-xs font-medium text-muted-foreground">MVP</p>
              <ul className="mt-3 space-y-2 text-sm text-muted-foreground">
                <li>Client profiles with allowlists (per-upstream)</li>
                <li>Full vs compact tool exposure modes</li>
                <li>Upstream presets + test connection</li>
                <li>Consent dialog for command execution</li>
              </ul>
            </div>
            <div className="rounded-xl border border-border bg-background/70 p-5 shadow-sm backdrop-blur">
              <p className="text-xs font-medium text-muted-foreground">Future</p>
              <ul className="mt-3 space-y-2 text-sm text-muted-foreground">
                <li>Optional sync/login for profiles and presets</li>
                <li>Safer secret storage (beyond plain JSON)</li>
                <li>Team sharing and audit-friendly activity logs</li>
              </ul>
            </div>
          </div>
        </section>

        <footer className="border-t border-border py-10">
          <div className="flex flex-col gap-2 text-sm text-muted-foreground sm:flex-row sm:items-center sm:justify-between">
            <p>© {new Date().getFullYear()} MCP Daddy</p>
            <div className="flex flex-wrap gap-x-4 gap-y-1">
              <a
                className="underline-offset-4 hover:underline"
                href="https://github.com/un/mcpdaddy"
                target="_blank"
                rel="noopener noreferrer"
              >
                GitHub
              </a>
              <a className="underline-offset-4 hover:underline" href="#roadmap">
                Roadmap
              </a>
              <a className="underline-offset-4 hover:underline" href="mailto:hello@example.com">
                Contact
              </a>
            </div>
          </div>
        </footer>
      </div>
    </main>
  );
}
