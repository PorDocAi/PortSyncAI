'use client'

import { useRouter } from 'next/navigation'
import { useEffect } from 'react'

export default function Layout({ children }: { children: React.ReactNode }) {
  const router = useRouter()
  useEffect(() => {
    const auth = localStorage.getItem('ps_admin_auth')
    if (!auth) router.replace('/signin')
  }, [router])
  return children
}
