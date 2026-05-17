import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

import App from '@/App'
import '@/index.css'

async function bootstrap(): Promise<void> {
  if (import.meta.env.VITE_ENABLE_MOCK === 'true') {
    const { worker } = await import('@/mocks/browser')
    await worker.start({ onUnhandledRequest: 'bypass' })
  }

  const rootEl = document.getElementById('root')
  if (!rootEl) {
    throw new Error('Root element #root not found')
  }
  createRoot(rootEl).render(
    <StrictMode>
      <App />
    </StrictMode>,
  )
}

void bootstrap()
