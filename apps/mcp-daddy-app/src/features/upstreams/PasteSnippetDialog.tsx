import { Button } from '@/components/ui/button';
import { useMemo, useState } from 'react';
import { cn } from '@/lib/utils';

type ParsedSnippet = {
  upstreamId: string;
  displayName: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  cwd?: string;
};

export function PasteSnippetDialog({
  open,
  onOpenChange,
  onSave,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSave: (parsed: ParsedSnippet) => Promise<void> | void;
}) {
  const [raw, setRaw] = useState('');
  const [parseError, setParseError] = useState<string | null>(null);
  const [parsed, setParsed] = useState<ParsedSnippet | null>(null);
  const preview = useMemo(() => {
    if (!parsed) return null;
    return {
      command: parsed.command,
      args: parsed.args,
      envKeys: Object.keys(parsed.env ?? {}),
      cwd: parsed.cwd,
    };
  }, [parsed]);

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
        <div className="w-full max-w-3xl rounded-2xl border bg-background shadow-xl">
          <div className="border-b p-5">
            <p className="text-xs font-medium text-muted-foreground">Add upstream</p>
            <h2 className="mt-2 text-lg font-semibold tracking-tight">Paste snippet</h2>
            <p className="mt-1 text-sm text-muted-foreground">
              Paste a JSON snippet with `command`, `args`, and optional `env`.
            </p>
          </div>

          <div className="p-5">
            <label className="grid gap-2">
              <span className="text-xs font-medium text-muted-foreground">snippet (json)</span>
              <textarea
                className={cn(
                  'min-h-40 w-full rounded-lg border border-input bg-background p-3 font-mono text-xs shadow-sm outline-none transition focus:border-ring'
                )}
                value={raw}
                onChange={(e) => {
                  setRaw(e.currentTarget.value);
                  setParseError(null);
                  setParsed(null);
                }}
                placeholder='{"command":"npx","args":["-y","@modelcontextprotocol/server-github"],"env":{"GITHUB_PERSONAL_ACCESS_TOKEN":"..."}}'
              />
            </label>

            {parseError ? <p className="mt-3 text-sm text-destructive">{parseError}</p> : null}

            {preview ? (
              <div className="mt-4 grid gap-3">
                <div className="rounded-lg border bg-background p-3">
                  <p className="text-xs font-medium text-muted-foreground">preview</p>
                  <pre className={cn('mt-2 font-mono text-xs', 'whitespace-pre-wrap break-all')}>
                    {preview.command}
                    {preview.args.length ? `\n${preview.args.map((a) => `- ${a}`).join('\n')}` : ''}
                  </pre>
                  <p className="mt-2 text-[11px] text-muted-foreground">
                    env keys: {preview.envKeys.length ? preview.envKeys.join(', ') : '(none)'}
                  </p>
                  {preview.cwd ? (
                    <p className="mt-1 text-[11px] text-muted-foreground">cwd: {preview.cwd}</p>
                  ) : null}
                </div>
              </div>
            ) : null}
          </div>

          <div className="flex items-center justify-between gap-2 border-t p-5">
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              Close
            </Button>

            <div className="flex items-center gap-2">
              <Button
                variant="secondary"
                onClick={() => {
                  const out = parseSnippet(raw);
                  if ('error' in out) {
                    setParseError(out.error);
                    setParsed(null);
                    return;
                  }
                  setParseError(null);
                  setParsed(out.value);
                }}
              >
                Parse
              </Button>
              <Button
                disabled={!parsed}
                onClick={async () => {
                  if (!parsed) return;
                  await onSave(parsed);
                  onOpenChange(false);
                }}
              >
                Save
              </Button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function parseSnippet(input: string): { value: ParsedSnippet } | { error: string } {
  let v: any;
  try {
    v = JSON.parse(input);
  } catch {
    return { error: 'Invalid JSON. Paste a JSON object with command/args/env.' };
  }

  if (v && typeof v === 'object' && v.mcpServers && typeof v.mcpServers === 'object') {
    const keys = Object.keys(v.mcpServers);
    if (keys.length === 0) return { error: 'mcpServers is empty.' };
    const k = keys[0];
    const entry = v.mcpServers[k];
    return parseServerEntry(entry, { upstreamId: k, displayName: k });
  }

  if (v && typeof v === 'object') {
    const upstreamId = typeof v.upstreamId === 'string' ? v.upstreamId : 'custom';
    const displayName = typeof v.displayName === 'string' ? v.displayName : upstreamId;
    return parseServerEntry(v, { upstreamId, displayName });
  }

  return { error: 'Snippet must be a JSON object.' };
}

function parseServerEntry(
  entry: any,
  base: { upstreamId: string; displayName: string }
): { value: ParsedSnippet } | { error: string } {
  const command = entry?.command;
  const args = entry?.args;
  const env = entry?.env;
  const cwd = entry?.cwd;

  if (typeof command !== 'string' || command.trim().length === 0) {
    return { error: 'Missing/invalid `command` (must be a non-empty string).' };
  }
  if (!Array.isArray(args) || !args.every((a) => typeof a === 'string')) {
    return { error: 'Missing/invalid `args` (must be an array of strings).' };
  }

  const outEnv: Record<string, string> = {};
  if (env != null) {
    if (typeof env !== 'object' || Array.isArray(env)) {
      return { error: 'Invalid `env` (must be an object of string -> string).' };
    }
    for (const [k, val] of Object.entries(env)) {
      if (typeof k !== 'string' || typeof val !== 'string') {
        return { error: 'Invalid `env` values (must be strings).' };
      }
      outEnv[k] = val;
    }
  }

  if (cwd != null && typeof cwd !== 'string') {
    return { error: 'Invalid `cwd` (must be a string if provided).' };
  }

  return {
    value: {
      upstreamId: base.upstreamId,
      displayName: base.displayName,
      command,
      args,
      env: outEnv,
      cwd: cwd ?? undefined,
    },
  };
}
