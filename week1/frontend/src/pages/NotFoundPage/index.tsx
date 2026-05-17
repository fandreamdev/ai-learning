import { Link } from 'react-router-dom'

export default function NotFoundPage() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center p-8">
      <h1 className="text-3xl font-semibold">404</h1>
      <p className="mt-2 text-gray-500">页面不存在</p>
      <Link to="/" className="mt-6 text-blue-600 underline">
        返回首页
      </Link>
    </main>
  )
}
