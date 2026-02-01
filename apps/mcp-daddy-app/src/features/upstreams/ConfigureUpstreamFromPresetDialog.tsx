import { Button } from '@/components/ui/button';
import type { UpstreamPreset } from './presets';
import { useEffect, useMemo, useState } from 'react';

export function ConfigureUpstreamFromPresetDialog({
  open,
  onOpenChange,
  preset,
  onContinue,
  onCancel,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  preset: UpstreamPreset;
  onContinue: (env: Record<string, string>) => void;
  onCancel?: () => void;
}) {
  const initial = useMemo(
    () =>
      Object.fromEntries(
        preset.requiredEnvKeys.map((k) => [k, ''])
      ) as Record<string, string>,
    [preset.requiredEnvKeys]
  );
  const [env, setEnv] = useState<Record<string, string>>(initial);

  useEffect(() => {
    setEnv(initial);
  }, [initial]);

  if (!open) return null;

  return (
    <div className="fixed inset-0 z-50">
      <button
        type="button"
        aria-label="Close"
        className="absolute inset-0 bg-black/50"
        onClick={() => {
          onCancel?.();
          onOpenChange(false);
        }}
      />

      <div className="absolute inset-0 flex items-center justify-center p-4">
        <div className="w-full max-w-2xl rounded-2xl border bg-background shadow-xl">
          <div className="border-b p-5">
            <p className="text-xs font-medium text-muted-foreground">Configure preset</p>
            <h2 className="mt-2 text-lg font-semibold tracking-tight">{preset.displayName}</h2>
            <p className="mt-1 text-sm text-muted-foreground">
              Required env values will be saved in your local config file for MVP.
            </p>
          </div>

          <div className="p-5">
            {preset.requiredEnvKeys.length === 0 ? (
              <p className="text-sm text-muted-foreground">No env variables required.</p>
            ) : (
              <div className="grid gap-3">
                {preset.requiredEnvKeys.map((k) => (
                  <label key={k} className="grid gap-1">
                    <span className="text-xs font-medium text-muted-foreground">{k}</span>
                    <input
                      type="password"
                      className="h-10 rounded-lg border border-input bg-background px-3 font-mono text-xs shadow-sm outline-none transition focus:border-ring"
                      value={env[k] ?? ''}
                      onChange={(e) => setEnv((prev) => ({ ...prev, [k]: e.currentTarget.value }))}
                      placeholder="Enter value"
                    />
                  </label>
                ))}
              </div>
            )}
          </div>

          <div className="flex items-center justify-end gap-2 border-t p-5">
            <Button
              variant="outline"
              onClick={() => {
                onCancel?.();
                onOpenChange(false);
              }}
            >
              Cancel
            </Button>
            <Button
              onClick={() => {
                onContinue(env);
                onOpenChange(false);
              }}
            >
              Continue
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
