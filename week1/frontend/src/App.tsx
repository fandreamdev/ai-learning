import { BrowserRouter, Route, Routes } from 'react-router-dom'

import NotFoundPage from '@/pages/NotFoundPage'
import TicketDetailPage from '@/pages/TicketDetailPage'
import TicketListPage from '@/pages/TicketListPage'

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route path="/" element={<TicketListPage />} />
        <Route path="/tickets/:id" element={<TicketDetailPage />} />
        <Route path="*" element={<NotFoundPage />} />
      </Routes>
    </BrowserRouter>
  )
}
