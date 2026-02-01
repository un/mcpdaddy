import type { ClientProfile, Upstream } from '../types';

export function ProfileDetail({
  profile,
  upstreams,
}: {
  profile: ClientProfile;
  upstreams: Upstream[];
}) {
  const allowed = new Set(profile.allowedUpstreamIds);

  return (
    <div className="rounded-2xl border bg-background/70 p-5 shadow-sm backdrop-blur">
      <p className="text-xs font-medium text-muted-foreground">Client profile detail (placeholder)</p>
      <h2 className="mt-2 text-lg font-semibold tracking-tight">{profile.displayName}</h2>

      <div className="mt-4 grid gap-3 text-sm">
        <div className="rounded-lg border bg-background p-3">
          <p className="text-xs font-medium text-muted-foreground">profileId</p>
          <p className="mt-1 font-mono text-xs">{profile.id}</p>
        </div>
        <div className="rounded-lg border bg-background p-3">
          <p className="text-xs font-medium text-muted-foreground">exposureMode</p>
          <p className="mt-1">{profile.exposureMode}</p>
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
                <span className="text-xs text-muted-foreground">
                  {allowed.has(u.id) ? 'allowed' : 'denied'}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
