import { createApi } from '@devup-api/fetch'

export const client = createApi({
  baseUrl: process.env.NEXT_PUBLIC_API_BASE_URL ?? 'http://localhost:8000/',
})
