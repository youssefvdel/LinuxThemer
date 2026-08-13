export function SkeletonCard() {
  return (
    <div className="card skeleton-card" aria-hidden="true">
      <div className="skel skel-thumb" />
      <div className="card-body">
        <div className="card-title-row">
          <div className="skel skel-title" />
          <div className="skel skel-swatches" />
        </div>
        <div className="skel skel-line" />
        <div className="skel skel-line short" />
        <div className="skel skel-meta" />
        <div className="card-footer">
          <div className="skel skel-stars" />
          <div className="skel skel-btn" />
        </div>
      </div>
    </div>
  );
}

export function SkeletonGrid({ count = 9 }: { count?: number }) {
  return (
    <div className="grid">
      {Array.from({ length: count }).map((_, i) => (
        <SkeletonCard key={i} />
      ))}
    </div>
  );
}
