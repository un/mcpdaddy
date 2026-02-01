import { Button } from '@/components/ui/button';
import { cn } from '@/lib/utils';

export type RunCommandSpec = {
  command: string;
  args: string[];
};

export function RunCommandConsentDialog({
  open,
  onOpenChange,
  spec,
  onApprove,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  spec: RunCommandSpec;
  onApprove: () => void;
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
          <div className="border-b p-5">
            <p className="text-xs font-medium text-muted-foreground">Consent required</p>
            <h2 className="mt-2 text-lg font-semibold tracking-tight">Run Command</h2>
            <p className="mt-1 text-sm text-muted-foreground">
              Review the exact command and arguments. Approving will allow this command to run.
            </p>
          </div>

          <div className="p-5">
            <div className="rounded-lg border bg-background p-3">
              <p className="text-xs font-medium text-muted-foreground">command</p>
              <pre className={cn('mt-2 font-mono text-xs', 'whitespace-pre-wrap break-all')}>
                {spec.command}
              </pre>
            </div>

            <div className="mt-3 rounded-lg border bg-background p-3">
              <p className="text-xs font-medium text-muted-foreground">args</p>
              {spec.args.length === 0 ? (
                <p className="mt-2 text-xs text-muted-foreground">(none)</p>
              ) : (
                <pre className={cn('mt-2 font-mono text-xs', 'whitespace-pre-wrap break-all')}>
                  {spec.args.map((a) => `- ${a}`).join('\n')}
                </pre>
              )}
            </div>
          </div>

          <div className="flex items-center justify-end gap-2 border-t p-5">
            <Button variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={() => {
                onOpenChange(false);
                onApprove();
              }}
            >
              Approve & Run
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
