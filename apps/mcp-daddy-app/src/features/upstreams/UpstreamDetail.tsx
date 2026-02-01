import type { Upstream } from '../types';
import { Button } from '@/components/ui/button';
import { useMemo, useState } from 'react';
import { UPSTREAM_PRESETS } from './presets';
import { RunCommandConsentDialog, type RunCommandSpec } from '../consent/RunCommandConsentDialog';

export function UpstreamDetail({ upstream }: { upstream: Upstream }) {
  const [consentOpen, setConsentOpen] = useState(false);
  const [ranCount, setRanCount] = useState(0);

  const sampleSpec: RunCommandSpec = useMemo(
    () => ({
      command: 'npx',
      args: ['-y', '@modelcontextprotocol/server-github'],
    }),
    []
  );

  return (
    <div className="rounded-2xl border bg-background/70 p-5 shadow-sm backdrop-blur">
      <p className="text-xs font-medium text-muted-foreground">Upstream detail (placeholder)</p>
      <h2 className="mt-2 text-lg font-semibold tracking-tight">{upstream.displayName}</h2>
      <div className="mt-4 grid gap-3 text-sm">
        <div className="rounded-lg border bg-background p-3">
          <p className="text-xs font-medium text-muted-foreground">run command consent</p>
          <div className="mt-2 flex items-center justify-between gap-3">
            <p className="text-xs text-muted-foreground">
              Runs approved: <span className="font-mono">{ranCount}</span>
            </p>
            <Button size="sm" variant="outline" onClick={() => setConsentOpen(true)}>
              Preview Dialog
            </Button>
          </div>
        </div>
        <div className="rounded-lg border bg-background p-3">
          <p className="text-xs font-medium text-muted-foreground">upstreamId</p>
          <p className="mt-1 font-mono text-xs">{upstream.id}</p>
        </div>
        <div className="rounded-lg border bg-background p-3">
          <p className="text-xs font-medium text-muted-foreground">status</p>
          <p className="mt-1">{upstream.status}</p>
        </div>
        <div className="rounded-lg border bg-background p-3">
          <p className="text-xs font-medium text-muted-foreground">command</p>
          <p className="mt-1 text-muted-foreground">(to be configured)</p>
        </div>
        <div className="rounded-lg border bg-background p-3">
          <p className="text-xs font-medium text-muted-foreground">env</p>
          <p className="mt-1 text-muted-foreground">(to be configured)</p>
        </div>

        <div className="rounded-lg border bg-background p-3">
          <p className="text-xs font-medium text-muted-foreground">presets</p>
          <div className="mt-2 grid gap-2">
            {UPSTREAM_PRESETS.map((p) => (
              <div key={p.id} className="rounded-md border bg-background px-3 py-2">
                <div className="flex items-center justify-between gap-3">
                  <p className="text-sm font-medium">{p.displayName}</p>
                  <p className="font-mono text-[10px] text-muted-foreground">{p.id}</p>
                </div>
                <p className="mt-1 text-xs text-muted-foreground">{p.description}</p>
                <p className="mt-2 text-[11px] text-muted-foreground">
                  {p.requiredEnvKeys.length === 0
                    ? 'required env: (none)'
                    : `required env: ${p.requiredEnvKeys.join(', ')}`}
                </p>
              </div>
            ))}
          </div>
        </div>
      </div>

      <RunCommandConsentDialog
        open={consentOpen}
        onOpenChange={setConsentOpen}
        spec={sampleSpec}
        onApprove={() => setRanCount((n) => n + 1)}
      />
    </div>
  );
}
