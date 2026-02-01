import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';
import { useEffect, useState } from 'react';

export type EditableProfile = {
  profileId: string;
  displayName: string;
  exposureMode: 'full' | 'compact';
  allowedUpstreamIds: string[];
};

export function EditProfileDialog({
  open,
  onOpenChange,
  mode,
  initial,
  onSave,
  onDelete,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  mode: 'create' | 'edit';
  initial: EditableProfile;
  onSave: (profile: EditableProfile) => Promise<void> | void;
  onDelete?: (profileId: string) => Promise<void> | void;
}) {
  const [draft, setDraft] = useState<EditableProfile>(initial);

  useEffect(() => {
    setDraft(initial);
  }, [initial]);

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
        <div className="w-full max-w-xl rounded-2xl border bg-background shadow-xl">
          <div className="border-b p-5">
            <p className="text-xs font-medium text-muted-foreground">Client profile</p>
            <h2 className="mt-2 text-lg font-semibold tracking-tight">
              {mode === 'create' ? 'New profile' : 'Edit profile'}
            </h2>
          </div>

          <div className="p-5">
            <div className="grid gap-3">
              <label className="grid gap-1">
                <span className="text-xs font-medium text-muted-foreground">profileId</span>
                <input
                  className={cn(
                    'h-10 rounded-lg border border-input bg-background px-3 font-mono text-xs shadow-sm outline-none transition focus:border-ring'
                  )}
                  value={draft.profileId}
                  disabled={mode === 'edit'}
                  onChange={(e) => setDraft((p) => ({ ...p, profileId: e.currentTarget.value }))}
                  placeholder="default"
                />
              </label>

              <label className="grid gap-1">
                <span className="text-xs font-medium text-muted-foreground">display name</span>
                <input
                  className={cn(
                    'h-10 rounded-lg border border-input bg-background px-3 text-sm shadow-sm outline-none transition focus:border-ring'
                  )}
                  value={draft.displayName}
                  onChange={(e) => setDraft((p) => ({ ...p, displayName: e.currentTarget.value }))}
                  placeholder="Work"
                />
              </label>

              <label className="grid gap-1">
                <span className="text-xs font-medium text-muted-foreground">exposure mode</span>
                <select
                  className="h-10 rounded-lg border border-input bg-background px-3 text-sm shadow-sm outline-none transition focus:border-ring"
                  value={draft.exposureMode}
                  onChange={(e) =>
                    setDraft((p) => ({
                      ...p,
                      exposureMode: e.currentTarget.value as 'full' | 'compact',
                    }))
                  }
                >
                  <option value="compact">compact</option>
                  <option value="full">full</option>
                </select>
              </label>
            </div>
          </div>

          <div className="flex items-center justify-between gap-2 border-t p-5">
            {mode === 'edit' && onDelete ? (
              <Button
                variant="destructive"
                onClick={async () => {
                  await onDelete(draft.profileId);
                  onOpenChange(false);
                }}
              >
                Delete
              </Button>
            ) : (
              <span />
            )}
            <div className="flex items-center gap-2">
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                Cancel
              </Button>
              <Button
                onClick={async () => {
                  await onSave(draft);
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
