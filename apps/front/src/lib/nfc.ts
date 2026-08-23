import { isTauri } from '@tauri-apps/api/core'

export type EquipmentScanPayload = {
  assignmentId: string
  equipmentCategory: string
  tagToken: string
  scannedAt: string
  device: {
    channel: 'phone-nfc' | 'browser-demo'
    platform: 'tauri-mobile' | 'web-preview'
  }
}

type ScanEquipmentTagInput = Pick<EquipmentScanPayload, 'assignmentId' | 'equipmentCategory'>

function toTagToken(bytes: readonly number[]) {
  return bytes.map((byte) => byte.toString(16).padStart(2, '0')).join('').toUpperCase()
}

export async function scanEquipmentTag({
  assignmentId,
  equipmentCategory,
}: ScanEquipmentTagInput): Promise<EquipmentScanPayload> {
  const scannedAt = new Date().toISOString()

  if (!isTauri()) {
    if (process.env.NODE_ENV === 'development') {
      return {
        assignmentId,
        equipmentCategory,
        tagToken: `DEMO-${equipmentCategory.replaceAll(' ', '-').toUpperCase()}`,
        scannedAt,
        device: { channel: 'browser-demo', platform: 'web-preview' },
      }
    }

    throw new Error('보호구 NFC는 PortSyncAI 모바일 앱에서만 확인할 수 있습니다.')
  }

  const { isAvailable, scan } = await import('@tauri-apps/plugin-nfc')
  if (!(await isAvailable())) {
    throw new Error('이 기기에서 NFC를 사용할 수 없습니다. NFC 설정과 기기 지원 여부를 확인해 주세요.')
  }

  const tag = await scan(
    { type: 'tag' },
    {
      keepSessionAlive: false,
      message: `${equipmentCategory} NFC 태그를 휴대폰 뒷면에 대주세요.`,
      successMessage: `${equipmentCategory} 태그를 읽었습니다.`,
    },
  )
  const tagToken = toTagToken(Array.from(tag.id))

  if (!tagToken) {
    throw new Error('NFC 태그 식별자를 읽지 못했습니다. 태그를 다시 대주세요.')
  }

  return {
    assignmentId,
    equipmentCategory,
    tagToken,
    scannedAt,
    device: { channel: 'phone-nfc', platform: 'tauri-mobile' },
  }
}
