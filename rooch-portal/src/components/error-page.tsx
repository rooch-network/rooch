// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
export default function ErrorPage() {
  return (
    <div className="flex items-center justify-center min-h-screen bg-gray-100">
      <div className="text-center p-10 bg-white shadow-xl rounded-lg">
        <h1 className="text-3xl font-bold text-gray-800 mb-4">404 Not Found</h1>
        <p className="text-gray-600 mb-8">Sorry, the page you are looking for does not exist.</p>
        <a
          href="/"
          className="inline-block px-6 py-2 text-sm font-semibold text-white bg-gray-600 rounded-full hover:bg-gray-500 transition-all"
        >
          Go Home
        </a>
      </div>
    </div>
  )
}
