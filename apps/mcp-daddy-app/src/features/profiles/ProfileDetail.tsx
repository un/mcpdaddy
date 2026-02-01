import type { ClientProfile, Upstream } from '../types';
import { Button } from '@/components/ui/button';

export function ProfileDetail({
  profile,
  upstreams,
  onEdit,
  onDelete,
}: {
  profile: ClientProfile;
  upstreams: Upstream[];
  onEdit?: () => void;
  onDelete?: () => void;
}) {
  const allowed = new Set(profile.allowedUpstreamIds);

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
