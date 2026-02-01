import type { Upstream } from '../types';

export function UpstreamDetail({ upstream }: { upstream: Upstream }) {
  return (
    <div className="rounded-2xl border bg-background/70 p-5 shadow-sm backdrop-blur">
      <p className="text-xs font-medium text-muted-foreground">Upstream detail (placeholder)</p>
      <h2 className="mt-2 text-lg font-semibold tracking-tight">{upstream.displayName}</h2>
      <div className="mt-4 grid gap-3 text-sm">
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
      </div>
    </div>
  );
}
