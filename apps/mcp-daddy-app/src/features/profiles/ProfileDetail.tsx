import type { ClientProfile, Upstream } from '../types';
import { Button } from '@/components/ui/button';
import { useMemo, useState } from 'react';

export function ProfileDetail({
  profile,
  upstreams,
  onEdit,
  onDelete,
  onUpdateAllowedUpstreamIds,
  onUpdateExposureMode,
}: {
  profile: ClientProfile;
  upstreams: Upstream[];
  onEdit?: () => void;
  onDelete?: () => void;
  onUpdateAllowedUpstreamIds?: (allowedUpstreamIds: string[]) => void;
  onUpdateExposureMode?: (exposureMode: 'full' | 'compact') => void;
}) {
  const allowed = new Set(profile.allowedUpstreamIds);
  const [copied, setCopied] = useState(false);
  const bridgeSnippet = useMemo(() => {
    return JSON.stringify(
      {
        mcpServers: {
          mcpdaddy: {
            command: 'mcp-daddy-bridge',
            args: ['--profile', profile.id],
          },
        },
      },
      null,
      2
    );
  }, [profile.id]);

  return (
    <div className="rounded-2xl border bg-background/70 p-5 shadow-sm backdrop-blur">
      <p className="text-xs font-medium text-muted-foreground">Client profile detail (placeholder)</p>
      <div className="mt-2 flex items-center justify-between gap-3">
        <h2 className="text-lg font-semibold tracking-tight">{profile.displayName}</h2>
        <div className="flex items-center gap-2">
          {onEdit ? (
            <Button size="sm" variant="outline" onClick={onEdit}>
              Edit
            </Button>
          ) : null}
          {onDelete ? (
            <Button size="sm" variant="destructive" onClick={onDelete}>
              Delete
            </Button>
          ) : null}
        </div>
      </div>

      <div className="mt-4 grid gap-3 text-sm">
        <div className="rounded-lg border bg-background p-3">
          <p className="text-xs font-medium text-muted-foreground">profileId</p>
          <p className="mt-1 font-mono text-xs">{profile.id}</p>
        </div>
        <div className="rounded-lg border bg-background p-3">
          <p className="text-xs font-medium text-muted-foreground">exposureMode</p>
          <select
            className="mt-2 h-9 w-full rounded-md border border-input bg-background px-2 text-sm shadow-sm outline-none transition focus:border-ring"
            value={profile.exposureMode}
            onChange={(e) => {
              onUpdateExposureMode?.(e.currentTarget.value as 'full' | 'compact');
            }}
          >
            <option value="compact">compact</option>
            <option value="full">full</option>
          </select>
        </div>
        <div className="rounded-lg border bg-background p-3">
          <p className="text-xs font-medium text-muted-foreground">allowed upstreams</p>
          <div className="mt-2 flex flex-col gap-2">
            {upstreams.map((u) => (
              <div
                key={u.id}
                className="flex items-center justify-between rounded-md border bg-background px-2 py-1"
              >
                <span className="text-sm">{u.displayName}</span>
                <label className="flex items-center gap-2 text-xs text-muted-foreground">
                  <input
                    type="checkbox"
                    checked={allowed.has(u.id)}
                    onChange={() => {
                      if (!onUpdateAllowedUpstreamIds) return;
                      const next = new Set(profile.allowedUpstreamIds);
                      if (next.has(u.id)) next.delete(u.id);
                      else next.add(u.id);
                      onUpdateAllowedUpstreamIds(Array.from(next));
                    }}
                  />
                  {allowed.has(u.id) ? 'allowed' : 'denied'}
                </label>
              </div>
            ))}
          </div>
        </div>

        <div className="rounded-lg border bg-background p-3">
          <div className="flex items-center justify-between gap-3">
            <p className="text-xs font-medium text-muted-foreground">connection snippet (stdio)</p>
            <Button
              size="sm"
              variant="outline"
              onClick={async () => {
                try {
                  await navigator.clipboard.writeText(bridgeSnippet);
                  setCopied(true);
                  setTimeout(() => setCopied(false), 1200);
                } catch {
                  // no-op
                }
              }}
            >
              {copied ? 'Copied' : 'Copy'}
            </Button>
          </div>
          <p className="mt-2 text-xs text-muted-foreground">
            Uses `--profile {profile.id}`.
          </p>
          <pre className="mt-3 max-h-64 overflow-auto rounded-lg border bg-background p-3 font-mono text-xs whitespace-pre-wrap break-all">
            {bridgeSnippet}
          </pre>
        </div>
      </div>
    </div>
  );
}
