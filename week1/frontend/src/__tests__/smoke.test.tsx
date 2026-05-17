import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import App from '@/App'

describe('App', () => {
  it('renders the list page placeholder at root', () => {
    render(<App />)
    expect(screen.getByText('ProjectAlpha')).toBeInTheDocument()
  })

  it('renders 404 placeholder for unknown route', () => {
    window.history.pushState({}, '', '/no-such-route')
    render(<App />)
    expect(screen.getByText('404')).toBeInTheDocument()
    window.history.pushState({}, '', '/')
  })
})
