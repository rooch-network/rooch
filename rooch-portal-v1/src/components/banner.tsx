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
          <span className="inline-flex p-1 me-2 bg-zinc-400 rounded-full dark:bg-zinc-600 w-5 h-5 items-center justify-center flex-shrink-0">
            <img src="/rooch_white_logo.svg" alt="rooc" className="w-3 h-3" />
            <span className="sr-only">Rooch</span>
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
          className="flex-shrink-0 inline-flex justify-center w-7 h-7 items-center text-zinc-400 hover:bg-zinc-200 hover:text-zinc-900 rounded-lg text-sm p-1.5 dark:hover:bg-zinc-600 dark:hover:text-white transition-all"
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
