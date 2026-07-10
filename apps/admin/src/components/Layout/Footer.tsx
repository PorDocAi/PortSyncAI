import { css, Text } from '@devup-ui/react'
import Link from 'next/link'

export function Footer() {
  return (
    <Text
      color="$caption"
      opacity="0.7"
      p={[4, null, 5]}
      typography="caption"
      wordBreak="keep-all"
    >
      © 2025 클라이언트명. All Rights Reserved. Designed & Developed by{' '}
      <Link
        className={css({
          color: '$caption',
          textDecoration: 'none',
          textUnderlineOffset: '4px',
          _hover: { color: '$caption', textDecoration: 'underline' },
        })}
        href="https://devfive.kr"
        target="_blank"
      >
        Devfive
      </Link>
    </Text>
  )
}
