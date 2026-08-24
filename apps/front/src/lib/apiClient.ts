import type { components, paths } from './api-types'

const baseUrl =
  process.env.NEXT_PUBLIC_API_BASE_URL ?? 'http://127.0.0.1:18090'

type JsonBody = Record<string, unknown>
type ResponseFor<Operation> = Operation extends { responses: infer Responses }
  ? Responses extends { 200: { content: { 'application/json': infer Body } } }
    ? Body
    : never
  : never

type BodyFor<Operation> = Operation extends {
  requestBody: { content: { 'application/json': infer Body } }
}
  ? Body
  : never

type RequestOptions = Omit<RequestInit, 'body' | 'headers'>

async function request<T>(path: string, options: RequestInit = {}): Promise<T> {
  const headers = new Headers(options.headers)
  headers.set('Accept', 'application/json')
  if (options.body) headers.set('Content-Type', 'application/json')
  if (typeof window !== 'undefined') {
    const token = localStorage.getItem('ps_token')
    if (token) headers.set('Authorization', `Bearer ${token}`)
  }
  const response = await fetch(`${baseUrl}${path}`, { ...options, headers })
  const body = await response.json().catch(() => undefined)
  if (!response.ok)
    throw new Error(
      typeof body === 'string'
        ? body
        : `API request failed (${response.status})`,
    )
  return body as T
}

export const apiClient = {
  signin: (body: BodyFor<paths['/auth/signin']['post']>) =>
    request<ResponseFor<paths['/auth/signin']['post']>>('/auth/signin', {
      method: 'POST',
      body: JSON.stringify(body),
    }),
  listEmployees: (options?: RequestOptions) =>
    request<ResponseFor<paths['/employees']['get']>>('/employees', {
      ...options,
      method: 'GET',
    }),
  createEmployee: (body: BodyFor<paths['/employees']['post']>) =>
    request<ResponseFor<paths['/employees']['post']>>('/employees', {
      method: 'POST',
      body: JSON.stringify(body),
    }),
  equipmentCheck: (body: BodyFor<paths['/equipment-checks']['post']>) =>
    request<ResponseFor<paths['/equipment-checks']['post']>>(
      '/equipment-checks',
      { method: 'POST', body: JSON.stringify(body) },
    ),
  issueGateTerminal: (body: BodyFor<paths['/gate/terminals']['post']>) =>
    request<ResponseFor<paths['/gate/terminals']['post']>>('/gate/terminals', {
      method: 'POST',
      body: JSON.stringify(body),
    }),
}

export type ApiComponents = components
export { baseUrl }
