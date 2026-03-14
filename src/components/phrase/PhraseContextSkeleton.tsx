import { Skeleton } from '../ui/skeleton';

export const PhraseContextSkeleton = () => {
  return (
    <div className="space-y-6">
      <div className="bg-blue-50/50 dark:bg-blue-900/20 border-l-4 border-blue-500 dark:border-blue-600 p-4 rounded space-y-3">
        <div className="flex items-center gap-2">
          <Skeleton className="h-4 w-4 rounded" />
          <Skeleton className="h-4 w-28" />
        </div>
        <div className="flex gap-2">
          <Skeleton className="h-6 w-20 rounded" />
          <Skeleton className="h-6 w-24 rounded" />
        </div>
        <Skeleton className="h-4 w-full" />
        <Skeleton className="h-4 w-full" />
        <Skeleton className="h-4 w-3/4" />
      </div>

      <div className="space-y-4">
        <div className="flex items-center gap-2">
          <Skeleton className="h-4 w-4 rounded" />
          <Skeleton className="h-5 w-28" />
        </div>
        <div className="space-y-4">
          {[1, 2].map((i) => (
            <div key={i}>
              <div className="flex items-center gap-2 mb-2">
                <Skeleton className="h-3 w-20" />
                <Skeleton className="h-5 w-16 rounded-full" />
              </div>
              <div className="pl-4 border-l-2 border-border">
                <Skeleton className="h-4 w-full" />
              </div>
            </div>
          ))}
        </div>
      </div>

      <div className="space-y-4">
        <div className="flex items-center gap-2">
          <Skeleton className="h-4 w-4 rounded" />
          <Skeleton className="h-5 w-36" />
        </div>
        <div className="p-4 bg-red-50/30 dark:bg-red-900/10 rounded border-l-4 border-red-300 dark:border-red-700 space-y-3">
          <Skeleton className="h-3 w-24" />
          <Skeleton className="h-4 w-full" />
          <div className="bg-green-50/50 dark:bg-green-900/20 p-3 rounded">
            <Skeleton className="h-4 w-full" />
          </div>
          <Skeleton className="h-3 w-5/6" />
        </div>
      </div>
    </div>
  );
};
