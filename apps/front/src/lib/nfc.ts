import { isTauri } from '@tauri-apps/api/core'

import { apiClient } from '@/lib/apiClient'

export type EquipmentScanPayload = {
  assignmentId: number
  equipmentCategory: string
  tagToken: string
  scannedAt: string
  device: {
    channel: 'phone-nfc' | 'browser-demo'
    platform: 'tauri-mobile' | 'web-preview'
  }
}

export type ScanEquipmentTagInput = Pick<
  EquipmentScanPayload,
  'assignmentId' | 'equipmentCategory'
>

export type TagResult = {
  payload: EquipmentScanPayload
  accepted: boolean
  reasonCode: string
  message: string | null
  progress: { required: number; satisfied: number; complete: boolean }
}

export const REASON_MESSAGES: Record<string, string> = {
  TAG_ACCEPTED: '보호구 확인 완료',
  TAG_ALREADY_ACCEPTED: '이미 확인된 보호구입니다.',
  TAG_NOT_REGISTERED: '등록되지 않은 태그입니다. 관리자에게 문의하세요.',
  EQUIPMENT_DAMAGED: '손상된 보호구입니다. 교체 후 다시 시도하세요.',
  EQUIPMENT_LOST: '분실 신고된 보호구입니다.',
  EQUIPMENT_BLOCKED: '사용이 차단된 보호구입니다.',
  EQUIPMENT_REPLACED: '교체 완료된 보호구입니다.',
  PERSONAL_EQUIPMENT_OWNER_MISMATCH: '다른 작업자의 보호구입니다.',
  SHARED_EQUIPMENT_IN_USE: '다른 작업이 사용 중인 공용 보호구입니다.',
  PPE_REQUIREMENT_ALREADY_SATISFIED: '이미 확인된 항목입니다.',
  WORK_ASSIGNMENT_NOT_FOUND: '배정된 작업을 찾을 수 없습니다.',
  NO_ACTIVE_WORK: '활성 작업이 없습니다.',
}

/**
 * NDEF 레코드에서 태그 토큰을 추출한다.
 * 지원 형식: 텍스트 레코드("demo-glove-token-001"),
 * URI 레코드("portsync://eq/demo-glove-token-001" — 접두사 제거).
 */
export function extractTokenFromRecords(
  records: readonly { tnf: number; payload: number[] }[],
): string {
  for (const record of records) {
    const text = String.fromCharCode(...record.payload)
    // URI 레코드: portsync://eq/<token>
    const uriMatch = text.match(/portsync:\/\/eq\/(.+)$/i)
    if (uriMatch) return uriMatch[1].trim()
    // NDEF 텍스트 레코드: 첫 바이트 = 언어코드 길이
    const langLen = record.payload[0] ?? 0
    const body = text.slice(1 + langLen).trim()
    if (/^[A-Za-z0-9:_-]{8,}$/.test(body)) return body
  }
  return ''
}

function toTagToken(bytes: readonly number[]) {
  return bytes
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('')
    .toUpperCase()
}

async function postEquipmentCheck(payload: EquipmentScanPayload) {
  const res = await apiClient.equipmentCheck({
    work_assignment_id: Number(payload.assignmentId),
    tag_token: payload.tagToken,
    client_scanned_at: payload.scannedAt,
    idempotency_key: crypto.randomUUID(),
  } as never)
  const body = res as {
    accepted?: boolean
    reason_code?: string
    message?: string | null
    progress?: { required: number; satisfied: number; complete: boolean }
  }
  const reasonCode = body.reason_code ?? 'UNKNOWN'
  return {
    accepted: body.accepted === true,
    reasonCode,
    message: REASON_MESSAGES[reasonCode] ?? body.message ?? reasonCode,
    progress: body.progress ?? { required: 0, satisfied: 0, complete: false },
  }
}

export async function scanEquipmentTag({
  assignmentId,
  equipmentCategory,
}: ScanEquipmentTagInput): Promise<TagResult> {
  const scannedAt = new Date().toISOString()

  let tagToken: string
  if (!isTauri()) {
    if (process.env.NODE_ENV !== 'development') {
      throw new Error(
        '보호구 NFC는 PortSyncAI 모바일 앱에서만 확인할 수 있습니다.',
      )
    }
    tagToken = `DEMO-${equipmentCategory.replaceAll(' ', '-').toUpperCase()}`
  } else {
    const { isAvailable, scan } = await import('@tauri-apps/plugin-nfc')
    if (!(await isAvailable())) {
      throw new Error(
        '이 기기에서 NFC를 사용할 수 없습니다. NFC 설정과 기기 지원 여부를 확인해 주세요.',
      )
    }

    const tag = await scan(
      { type: 'tag' },
      {
        keepSessionAlive: false,
        message: `${equipmentCategory} NFC 태그를 휴대폰 뒷면에 대주세요.`,
        successMessage: `${equipmentCategory} 태그를 읽었습니다.`,
      },
    )
    tagToken = extractTokenFromRecords(tag.records ?? [])
    if (!tagToken) {
      throw new Error(
        '태그에 유효한 보호구 토큰이 없습니다. 관리자에게 등록된 태그인지 확인해 주세요.',
      )
    }
  }

  const payload: EquipmentScanPayload = {
    assignmentId,
    equipmentCategory,
    tagToken,
    scannedAt,
    device: isTauri()
      ? { channel: 'phone-nfc', platform: 'tauri-mobile' }
      : { channel: 'browser-demo', platform: 'web-preview' },
  }

  // 서버에 태깅 결과 전송 — accepted/reason_code/progress는 서버가 판정
  const result = await postEquipmentCheck(payload)
  if (!result.accepted && result.reasonCode !== 'TAG_ALREADY_ACCEPTED') {
    throw Object.assign(new Error(result.message), { tagResult: result })
  }
  return { ...result, payload }
}
