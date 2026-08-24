const BASE = 'http://127.0.0.1:18090'
let pass = 0, fail = 0
function check(name, ok, detail = '') {
  if (ok) { pass++; console.log(`PASS ${name}${detail ? ' — ' + detail : ''}`) }
  else { fail++; console.log(`FAIL ${name}${detail ? ' — ' + detail : ''}`) }
}

// 1. Admin signin
const admin = await fetch(`${BASE}/auth/signin`, {
  method: 'POST', headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ email: 'admin@port.test', password: 'admin1234' }),
}).then(r => r.json())
check('admin signin', !!admin.token, `role=${admin.system_role}`)

// 2. Create employee
const empNo = `E${Date.now().toString().slice(-6)}`
const created = await fetch(`${BASE}/employees`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${admin.token}` },
  body: JSON.stringify({
    employee_number: empNo, name: '스모크작업자', email: `${empNo.toLowerCase()}@smoke.test`,
    password: 'Smoke1234!', department_id: 1, job_role_id: 1,
    hire_date: new Date().toISOString().slice(0, 10),
  }),
})
check('employee create', created.status === 201 || created.status === 200, `status=${created.status}`)

// 3. Worker (pre-seeded with v2 assignment) signin
const worker = await fetch(`${BASE}/auth/signin`, {
  method: 'POST', headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({ email: 'worker@port.test', password: 'worker1234' }),
}).then(r => r.json())
check('worker signin (seeded)', !!worker.token)

// 4. Equipment tag with valid token
const tag = await fetch(`${BASE}/equipment-checks`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${worker.token}` },
  body: JSON.stringify({
    work_assignment_id: 1, tag_token: 'eqt_test_token_worker',
    client_scanned_at: new Date().toISOString(),
    idempotency_key: crypto.randomUUID(),
  }),
})
const tagBody = await tag.json()
check('nfc tag accepted', tagBody.accepted === true, `reason=${tagBody.reason_code} progress=${JSON.stringify(tagBody.progress)}`)

// 5. Replay same idempotency key → identical response
const replay = await fetch(`${BASE}/equipment-checks`, {
  method: 'POST',
  headers: { 'Content-Type': 'application/json', Authorization: `Bearer ${worker.token}` },
  body: JSON.stringify({
    work_assignment_id: 1, tag_token: 'eqt_test_token_worker',
    client_scanned_at: new Date().toISOString(),
    idempotency_key: crypto.randomUUID(), // new key → TAG_ALREADY_ACCEPTED path
  }),
}).then(r => r.json())
check('duplicate tag replay', replay.accepted === true && replay.reason_code === 'TAG_ALREADY_ACCEPTED', `reason=${replay.reason_code}`)

console.log(`\n${pass} passed, ${fail} failed`)
process.exit(fail > 0 ? 1 : 0)
