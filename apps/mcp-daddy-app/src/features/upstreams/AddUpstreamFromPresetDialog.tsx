import type { UpstreamPreset } from './presets';
import { UPSTREAM_PRESETS } from './presets';
import { Button } from '@/components/ui/button';

export function AddUpstreamFromPresetDialog({
  open,
  onOpenChange,
  onSelectPreset,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSelectPreset: (preset: UpstreamPreset) => void;
}) {
  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50">
      <button
        type="button"
        aria-label="Close"
        className="absolute inset-0 bg-black/50"
        onClick={() => onOpenChange(false)}
      />

      <div className="absolute inset-0 flex items-center justify-center p-4">
        <div className="w-full max-w-2xl rounded-2xl border bg-background shadow-xl">
          <div className="flex items-start justify-between gap-3 border-b p-5">
            <div>
              <p className="text-xs font-medium text-muted-foreground">Add upstream</p>
              <h2 className="mt-2 text-lg font-semibold tracking-tight">From preset</h2>
              <p className="mt-1 text-sm text-muted-foreground">
                Pick a preset. MCP Daddy will save it to your local config.
              </p>
            </div>
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              Close
            </Button>
          </div>

          <div className="p-5">
            <div className="grid gap-3">
              {UPSTREAM_PRESETS.map((p) => (
                <div key={p.id} className="rounded-xl border bg-background p-4">
                  <div className="flex items-start justify-between gap-4">
                    <div>
                      <p className="text-sm font-semibold">{p.displayName}</p>
                      <p className="mt-1 text-xs text-muted-foreground">{p.description}</p>
                      <p className="mt-2 font-mono text-[11px] text-muted-foreground">
                        {p.command} {p.args.join(' ')}
                      </p>
                      <p className="mt-2 text-[11px] text-muted-foreground">
                        required env: {p.requiredEnvKeys.length ? p.requiredEnvKeys.join(', ') : '(none)'}
                      </p>
                    </div>
                    <Button
                      size="sm"
                      onClick={() => {
                        onSelectPreset(p);
                        onOpenChange(false);
                      }}
                    >
                      Use
                    </Button>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
