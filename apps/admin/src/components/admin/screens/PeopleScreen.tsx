'use client'

import { Button as UiButton, Input as UiInput } from '@devup-ui/react'
import { apiClient } from '@/lib/apiClient'
import { useEffect, useMemo, useState } from 'react'

type Employee = {
  employee_id: number
  employee_number: string
  name: string
  email: string
  system_role: string
  status: string
  position?: string | null
}

export function PeopleScreen() {
  const [query, setQuery] = useState('')
  const [selected, setSelected] = useState<Employee | null>(null)
  const [employees, setEmployees] = useState<Employee[]>([])
  const [loading, setLoading] = useState(true)
  const [showCreate, setShowCreate] = useState(false)
  const [createError, setCreateError] = useState('')

  const fetchList = async () => {
    try {
      const res = await apiClient.listEmployees()
      setEmployees(res as Employee[])
    } catch { /* 401 handled by guard */ }
    finally { setLoading(false) }
  }

  useEffect(() => { void fetchList() }, [])

  const filtered = useMemo(
    () => employees.filter((e) => `${e.employee_number} ${e.name} ${e.email}`.includes(query)),
    [employees, query],
  )

  const handleCreate = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    setCreateError('')
    const fd = new FormData(e.currentTarget)
    try {
      await apiClient.createEmployee({
        employee_number: String(fd.get('employee_number')),
        name: String(fd.get('name')),
        email: String(fd.get('email')),
        password: String(fd.get('password')),
        department_id: Number(fd.get('department_id')) || 1,
        job_role_id: Number(fd.get('job_role_id')) || 1,
        hire_date: String(fd.get('hire_date')) || new Date().toISOString().slice(0, 10),
      })
      setShowCreate(false)
      void fetchList()
    } catch (err) {
      setCreateError(err instanceof Error ? err.message : '생성 실패')
    }
  }

  return (
    <div className="admin-screen">
      <header className="screen-heading">
        <div><p className="admin-overline">PEOPLE / 05</p><h1>직원 관리</h1><p>계정, 권한, 소속과 사원증 UID를 관리합니다.</p></div>
        <UiButton onClick={() => setShowCreate((v) => !v)} type="button">직원 등록</UiButton>
      </header>
      <div className="admin-toolbar"><UiInput onChange={(event) => setQuery(event.target.value)} placeholder="이름·사번·부서 검색" value={query} /></div>
      {showCreate && (
        <form onSubmit={handleCreate} style={{ display:'grid', gap:12, maxWidth:420, margin:'16px 0', padding:16, border:'1px solid #ddd' }}>
          <UiInput name="employee_number" placeholder="사번 (예: W3001)" required />
          <UiInput name="name" placeholder="이름" required />
          <UiInput name="email" type="email" placeholder="이메일" required />
          <UiInput name="password" type="password" placeholder="임시 비밀번호" required />
          <input type="hidden" name="department_id" value={1} />
          <input type="hidden" name="job_role_id" value={1} />
          <input name="hire_date" placeholder="입사일 (YYYY-MM-DD)" defaultValue={new Date().toISOString().slice(0,10)} required />
          {createError && <p style={{ color:'#e53e3e' }}>{createError}</p>}
          <UiButton type="submit">등록</UiButton>
        </form>
      )}
      {loading ? (
        <p>로딩 중…</p>
      ) : (
        <div className="master-detail">
          <section className="data-region">
            <div className="data-head people-grid"><span>사번</span><span>이름</span><span>이메일</span><span>권한</span><span>상태</span></div>
            {filtered.map((emp) => (
              <div
                className={selected?.employee_id === emp.employee_id ? 'data-row people-grid is-selected' : 'data-row people-grid'}
                key={emp.employee_id}
                onClick={() => setSelected(emp)}
                role="button"
                tabIndex={0}
              >
                {[emp.employee_number, emp.name, emp.email, emp.system_role, emp.status].map((v, i) => (
                  <span className={i === 0 ? 'mono' : ''} key={`${emp.employee_id}-${i}`}>{v}</span>
                ))}
              </div>
            ))}
          </section>
          <aside className="detail-rail">
            {selected && (
              <>
                <div className="detail-rail__identity">
                  <span className="mono">{selected.employee_number}</span><strong>{selected.name}</strong>
                  <p>{selected.position ?? '—'} · {selected.email}</p>
                </div>
                <dl>
                  <div><dt>시스템 권한</dt><dd>{selected.system_role}</dd></div>
                  <div><dt>계정 상태</dt><dd>{selected.status}</dd></div>
                </dl>
              </>
            )}
          </aside>
        </div>
      )}
    </div>
  )
}

export function ScreenHeading({ code, title, description, action }: { code: string; title: string; description: string; action?: string }) {
  return (
    <header className="screen-heading">
      <div><p className="admin-overline">{code}</p><h1>{title}</h1><p>{description}</p></div>
      {action && <UiButton type="button">{action}</UiButton>}
    </header>
  )
}
