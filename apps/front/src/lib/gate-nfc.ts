'use client'

import { isTauri } from '@tauri-apps/api/core'

/** 게이트 단말용 사원증 NFC 스캔. UID(hex) 또는 NDEF 텍스트(EMP-... 형식)를 반환한다. */
export async function scanBadgeUid(): Promise<string> {
  if (!isTauri()) {
    throw new Error('NFC는 모바일 앱에서만 사용할 수 있습니다. 브라우저에서는 직접 입력해 주세요.')
  }
  const { isAvailable, scan } = await import('@tauri-apps/plugin-nfc')
  if (!(await isAvailable())) {
    throw new Error('이 기기에서 NFC를 사용할 수 없습니다.')
  }

  const tag = await scan(
    { type: 'ndef' },
    {
      keepSessionAlive: false,
      message: '사원증을 단말기 뒷면에 대주세요.',
      successMessage: '사원증을 읽었습니다.',
    },
  )

  // NDEF 텍스트 레코드 우선: 언어코드 바 건너뛰고 본문
  for (const record of tag.records ?? []) {
    const text = String.fromCharCode(...record.payload)
    const langLen = record.payload[0] ?? 0
    const body = text.slice(1 + langLen).trim()
    if (/^(EMP|emp)[A-Za-z0-9-_]*$/.test(body)) return body
  }
  // NDEF 없으면 태그 UID hex
  return (tag.id ?? [])
    .map((b: number) => b.toString(16).padStart(2, '0'))
    .join('')
    .toUpperCase()
}
