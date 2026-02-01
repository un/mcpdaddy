import { cn } from '@/lib/utils';

import type { ClientProfile } from '../types';

export function ProfilesList({
  profiles,
  selectedProfileId,
  onSelect,
}: {
  profiles: ClientProfile[];
  selectedProfileId: string | null;
  onSelect: (id: string) => void;
}) {
  return (
    <div className="p-2">
      {profiles.map((p) => {
        const isSelected = p.id === selectedProfileId;
        return (
          <button
            key={p.id}
            type="button"
            onClick={() => onSelect(p.id)}
            className={cn(
              'w-full rounded-lg border px-3 py-2 text-left text-sm transition',
              isSelected
                ? 'border-ring bg-accent text-accent-foreground'
                : 'border-border bg-background hover:bg-accent'
            )}
          >
            <div className="flex items-center justify-between gap-3">
              <span className="font-medium">{p.displayName}</span>
              <span className="text-xs text-muted-foreground">{p.exposureMode}</span>
            </div>
            <div className="mt-1 text-xs text-muted-foreground">{p.id}</div>
          </button>
        );
      })}
    </div>
  );
}
