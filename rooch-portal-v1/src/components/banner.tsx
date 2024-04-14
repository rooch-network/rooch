interface BannerProps {
  onClose: () => void
}

export const Banner: React.FC<BannerProps> = ({ onClose }) => {
  return (
    <div
      id="sticky-banner"
      tabIndex={-1}
      className="fixed top-0 start-0 z-50 flex justify-between w-full p-1 border-b border-zinc-100 bg-zinc-100 dark:bg-zinc-800 dark:border-zinc-800 inset-x-0"
    >
      <div className="flex items-center mx-auto">
        <p className="flex items-center text-sm font-normal text-zinc-500 dark:text-zinc-400">
          <span className="inline-flex p-1 me-3 bg-zinc-200 rounded-full dark:bg-zinc-600 w-6 h-6 items-center justify-center flex-shrink-0">
            <svg
              className="w-3 h-3 text-zinc-500 dark:text-zinc-400"
              aria-hidden="true"
              xmlns="http://www.w3.org/2000/svg"
              fill="currentColor"
              viewBox="0 0 18 19"
            >
              <path d="M15 1.943v12.114a1 1 0 0 1-1.581.814L8 11V5l5.419-3.871A1 1 0 0 1 15 1.943ZM7 4H2a2 2 0 0 0-2 2v4a2 2 0 0 0 2 2v5a2 2 0 0 0 2 2h1a2 2 0 0 0 2-2V4ZM4 17v-5h1v5H4ZM16 5.183v5.634a2.984 2.984 0 0 0 0-5.634Z" />
            </svg>
            <span className="sr-only">Light bulb</span>
          </span>
          <span>
            Rooch BTC Layer2 is comming soon{' '}
            <a
              href="https://rooch.network/"
              className="inline font-medium text-blue-600 underline dark:text-blue-500 underline-offset-2 decoration-600 dark:decoration-500 decoration-solid hover:text-blue-500 dark:hover:text-blue-400 transition-all"
            >
              Learn more
            </a>
            .
          </span>
        </p>
      </div>
      <div className="flex items-center">
        <button
          onClick={onClose}
          type="button"
          className="flex-shrink-0 inline-flex justify-center w-7 h-7 items-center text-zinc-400 hover:bg-zinc-200 hover:text-zinc-900 rounded-lg text-sm p-1.5 dark:hover:bg-zinc-600 dark:hover:text-white"
        >
          <svg
            className="w-2.5 h-2.5"
            aria-hidden="true"
            xmlns="http://www.w3.org/2000/svg"
            fill="none"
            viewBox="0 0 14 14"
          >
            <path
              stroke="currentColor"
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="m1 1 6 6m0 0 6 6M7 7l6-6M7 7l-6 6"
            />
          </svg>
          <span className="sr-only">Close</span>
        </button>
      </div>
    </div>
  )
}
