type BrandLockupProps = {
  className?: string
  compact?: boolean
}

export function BrandLockup({ className = '', compact = false }: BrandLockupProps) {
  return (
    <div className={`wordmark ${className}`.trim()} aria-label="PortSyncAI">
      <svg aria-hidden="true" className="brand-symbol" viewBox="0 0 32 32">
        <path d="M5 7h15l5 5-5 5H10" />
        <path d="M27 25H12l-5-5 5-5h10" />
        <path d="M10 10h8M14 14h8M10 22h8" />
      </svg>
      {!compact && <strong>PortSyncAI</strong>}
    </div>
  )
}
