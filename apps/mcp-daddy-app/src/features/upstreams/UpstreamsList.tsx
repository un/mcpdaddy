import { cn } from '@/lib/utils';

import type { Upstream } from '../types';

export function UpstreamsList({
  upstreams,
  selectedUpstreamId,
  onSelect,
}: {
  upstreams: Upstream[];
  selectedUpstreamId: string | null;
  onSelect: (id: string) => void;
}) {
  return (
    <div className="p-2">
      {upstreams.map((u) => {
        const isSelected = u.id === selectedUpstreamId;
        return (
          <button
            key={u.id}
            type="button"
            onClick={() => onSelect(u.id)}
            className={cn(
              'w-full rounded-lg border px-3 py-2 text-left text-sm transition',
              isSelected
                ? 'border-ring bg-accent text-accent-foreground'
                : 'border-border bg-background hover:bg-accent'
            )}
          >
            <div className="flex items-center justify-between gap-3">
              <span className="font-medium">{u.displayName}</span>
              <span className="text-xs text-muted-foreground">{u.status}</span>
            </div>
            <div className="mt-1 text-xs text-muted-foreground">{u.id}</div>
          </button>
        );
      })}
    </div>
  );
}
