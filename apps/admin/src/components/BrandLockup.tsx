type BrandLockupProps = {
  className?: string
  compact?: boolean
  suffix?: string
}

export function BrandLockup({ className = '', compact = false, suffix }: BrandLockupProps) {
  return (
    <div className={`admin-wordmark ${className}`.trim()} aria-label={`PortSyncAI${suffix ? ` ${suffix}` : ''}`}>
      <svg aria-hidden="true" className="brand-symbol" viewBox="0 0 32 32">
        <path d="M5 7h15l5 5-5 5H10" />
        <path d="M27 25H12l-5-5 5-5h10" />
        <path d="M10 10h8M14 14h8M10 22h8" />
      </svg>
      {!compact && <strong>PortSyncAI{suffix ? ` ${suffix}` : ''}</strong>}
    </div>
  )
}
