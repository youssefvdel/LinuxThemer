function StarShape() {
  return (
    <svg width="12" height="12" viewBox="0 0 24 24" aria-hidden="true">
      <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
    </svg>
  );
}

export function Stars({ rating }: { rating: number }) {
  return (
    <span className="stars" title={`${rating.toFixed(1)} / 5`}>
      <span className="stars-track">
        {[1, 2, 3, 4, 5].map((i) => {
          const fill = Math.max(0, Math.min(1, rating - (i - 1)));
          return (
            <span key={i} className="star-cell">
              <span className="star-bg">
                <StarShape />
              </span>
              {fill > 0 && (
                <span className="star-fill" style={{ width: `${fill * 100}%` }}>
                  <StarShape />
                </span>
              )}
            </span>
          );
        })}
      </span>
      <span className="stars-num">{rating.toFixed(1)}</span>
    </span>
  );
}
