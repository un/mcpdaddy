import { useEffect, useMemo, useState } from "react";
import reactLogo from "./assets/react.svg";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import { ProfileDetail } from "@/features/profiles/ProfileDetail";
import { ProfilesList } from "@/features/profiles/ProfilesList";
import type { ClientProfile, Upstream } from "@/features/types";
import { UpstreamDetail } from "@/features/upstreams/UpstreamDetail";
import { UpstreamsList } from "@/features/upstreams/UpstreamsList";
import { AddUpstreamFromPresetDialog } from "@/features/upstreams/AddUpstreamFromPresetDialog";
import { ConfigureUpstreamFromPresetDialog } from "@/features/upstreams/ConfigureUpstreamFromPresetDialog";
import { RunCommandConsentDialog, type RunCommandSpec } from "@/features/consent/RunCommandConsentDialog";
import type { UpstreamPreset } from "@/features/upstreams/presets";

function App() {
  const [config, setConfig] = useState<{
    schemaVersion: number;
    upstreamServers: { upstreamId: string; displayName: string }[];
    clientProfiles: {
      profileId: string;
      displayName: string;
      exposureMode: "full" | "compact";
      allowedUpstreamIds: string[];
    }[];
  } | null>(null);

  useEffect(() => {
    void invoke("config_get")
      .then((cfg) => setConfig(cfg as any))
      .catch(() => setConfig(null));
  }, []);

  const upstreams: Upstream[] = useMemo(() => {
    if (!config) return [];
    return config.upstreamServers.map((u) => ({
      id: u.upstreamId,
      displayName: u.displayName,
      status: "stopped",
    }));
  }, [config]);

  const profiles: ClientProfile[] = useMemo(() => {
    if (!config) return [];
    return config.clientProfiles.map((p) => ({
      id: p.profileId,
      displayName: p.displayName,
      exposureMode: p.exposureMode,
      allowedUpstreamIds: p.allowedUpstreamIds,
    }));
  }, [config]);

  const [selection, setSelection] = useState<{ kind: "upstream" | "profile"; id: string } | null>(
    null
  );

  useEffect(() => {
    if (!selection && upstreams[0]) {
      setSelection({ kind: "upstream", id: upstreams[0].id });
    }
  }, [selection, upstreams]);

  const [addPresetOpen, setAddPresetOpen] = useState(false);
  const [pendingPreset, setPendingPreset] = useState<UpstreamPreset | null>(null);
  const [pendingEnv, setPendingEnv] = useState<Record<string, string> | null>(null);
  const [configureOpen, setConfigureOpen] = useState(false);
  const [consentOpen, setConsentOpen] = useState(false);

  const pendingConsentSpec: RunCommandSpec | null = useMemo(() => {
    if (!pendingPreset) return null;
    return { command: pendingPreset.command, args: pendingPreset.args };
  }, [pendingPreset]);

  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  async function greet() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    setGreetMsg(await invoke("greet", { name }));
  }

  return (
    <main className="min-h-full bg-[radial-gradient(60rem_35rem_at_10%_-10%,rgba(14,116,144,0.18),transparent_60%),radial-gradient(55rem_35rem_at_100%_0%,rgba(234,88,12,0.12),transparent_60%),linear-gradient(180deg,hsl(var(--background)),hsl(var(--muted)))]">
      <div className="grid min-h-full grid-cols-1 md:grid-cols-[16rem_minmax(0,1fr)_18rem]">
        <aside className="border-b bg-background/70 backdrop-blur md:border-b-0 md:border-r">
          <div className="flex items-center justify-between gap-3 px-4 py-3">
            <div>
              <p className="text-xs font-medium text-muted-foreground">Upstreams</p>
              <p className="text-sm font-semibold tracking-tight">Local servers</p>
            </div>
            <Button size="sm" variant="outline" type="button" onClick={() => setAddPresetOpen(true)}>
              Add
            </Button>
          </div>
          <Separator />
          <ScrollArea className="h-[calc(100dvh-56px)] md:h-dvh">
            <UpstreamsList
              upstreams={upstreams}
              selectedUpstreamId={selection?.kind === "upstream" ? selection.id : null}
              onSelect={(id) => setSelection({ kind: "upstream", id })}
            />
          </ScrollArea>
        </aside>

        <section className="min-w-0">
          <div className="flex items-center justify-between gap-4 px-6 py-4">
            <div>
              <p className="text-xs font-medium text-muted-foreground">MCP Daddy</p>
              <h1 className="mt-1 text-lg font-semibold tracking-tight">Details</h1>
            </div>
            <div className="flex items-center gap-2">
              <a href="https://tauri.app" target="_blank" rel="noreferrer">
                <img src="/tauri.svg" className="h-6 w-6" alt="Tauri" />
              </a>
              <a href="https://vite.dev" target="_blank" rel="noreferrer">
                <img src="/vite.svg" className="h-6 w-6" alt="Vite" />
              </a>
              <a href="https://react.dev" target="_blank" rel="noreferrer">
                <img src={reactLogo} className="h-6 w-6" alt="React" />
              </a>
            </div>
          </div>
          <Separator />
          <div className="px-6 py-6">
            {!selection ? (
              <div className="rounded-2xl border bg-background/70 p-5 shadow-sm backdrop-blur">
                <p className="text-xs font-medium text-muted-foreground">No selection</p>
                <p className="mt-2 text-sm text-muted-foreground">
                  Add an upstream to get started, or select a profile.
                </p>
              </div>
            ) : selection.kind === "upstream" ? (
              <UpstreamDetail upstream={upstreams.find((u) => u.id === selection.id) ?? upstreams[0]} />
            ) : (
              <ProfileDetail
                profile={profiles.find((p) => p.id === selection.id) ?? profiles[0]}
                upstreams={upstreams}
              />
            )}

            <div className="mt-4 rounded-2xl border bg-background/70 p-5 shadow-sm backdrop-blur">
              <p className="text-xs font-medium text-muted-foreground">Tauri command demo (placeholder)</p>
              <form
                className="mt-3 flex flex-col gap-3 sm:flex-row"
                onSubmit={(e) => {
                  e.preventDefault();
                  greet();
                }}
              >
                <input
                  className="h-10 w-full rounded-lg border border-input bg-background px-3 text-sm shadow-sm outline-none transition focus:border-ring"
                  onChange={(e) => setName(e.currentTarget.value)}
                  placeholder="Enter a name..."
                />
                <Button type="submit">Greet</Button>
              </form>

              <p className="mt-3 min-h-6 text-sm text-muted-foreground">{greetMsg}</p>
            </div>
          </div>
        </section>

        <aside className="border-t bg-background/70 backdrop-blur md:border-l md:border-t-0">
          <div className="flex items-center justify-between gap-3 px-4 py-3">
            <div>
              <p className="text-xs font-medium text-muted-foreground">Client Profiles</p>
              <p className="text-sm font-semibold tracking-tight">Per-client rules</p>
            </div>
            <Button size="sm" variant="outline" type="button">
              New
            </Button>
          </div>
          <Separator />
          <ScrollArea className="h-[calc(100dvh-56px)] md:h-dvh">
            <ProfilesList
              profiles={profiles}
              selectedProfileId={selection?.kind === "profile" ? selection.id : null}
              onSelect={(id) => setSelection({ kind: "profile", id })}
            />
          </ScrollArea>
        </aside>
      </div>

      <AddUpstreamFromPresetDialog
        open={addPresetOpen}
        onOpenChange={setAddPresetOpen}
        onSelectPreset={(preset) => {
          setPendingPreset(preset);
          setConfigureOpen(true);
        }}
      />

      {pendingPreset ? (
        <ConfigureUpstreamFromPresetDialog
          open={configureOpen}
          onOpenChange={setConfigureOpen}
          preset={pendingPreset}
          onContinue={(env) => {
            setPendingEnv(env);
            setConsentOpen(true);
          }}
          onCancel={() => {
            setPendingPreset(null);
            setPendingEnv(null);
          }}
        />
      ) : null}

      {pendingPreset && pendingConsentSpec ? (
        <RunCommandConsentDialog
          open={consentOpen}
          onOpenChange={(o) => {
            setConsentOpen(o);
            if (!o) setPendingPreset(null);
            if (!o) setPendingEnv(null);
          }}
          spec={pendingConsentSpec}
          onApprove={async () => {
            if (!pendingEnv) return;
            const cfg = (await invoke("config_add_upstream_from_preset", {
              input: {
                upstreamId: pendingPreset.id,
                displayName: pendingPreset.displayName,
                command: pendingPreset.command,
                args: pendingPreset.args,
                env: pendingEnv,
              },
            })) as any;
            setConfig(cfg);
          }}
        />
      ) : null}
    </main>
  );
}

export default App;
