import { css } from '@devup-ui/react'
import Link from 'next/link'

import { HeaderButton } from '../Buttons/HeaderButton'

export function HeaderButtonContainer() {
  return (
    <>
      <Link
        className={css({ flex: 1 })}
        href="https://pf.kakao.com/_YUeZn"
        target="_blank"
      >
        <HeaderButton>유지보수 문의</HeaderButton>
      </Link>
      <Link
        className={css({ flex: 1 })}
        href="https://devfive.kr"
        target="_blank"
      >
        <HeaderButton>홈페이지 바로가기</HeaderButton>
      </Link>
    </>
  )
}
