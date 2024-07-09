import Image from 'next/image'
import { useState } from 'react'

interface FilterButtonInterface {
  options: Array<{
    id: string
    text: string
    avatar: string
  }>
  onClick: Function
}

export function FilterButton({ options, onClick }: FilterButtonInterface) {
  const [buttonDefault, SetButtonDefault] = useState(options[0])

  return (
    <div className="hs-dropdown relative inline-flex">
      <button
        id="hs-dropdown-auto-close-outside"
        type="button"
        className="hs-dropdown-toggle py-3 px-4 inline-flex justify-center items-center gap-2 rounded-md border font-medium bg-zinc-50 text-zinc-700 border-zinc-300 shadow-sm align-middle hover:bg-zinc-200 transition-all text-sm dark:bg-black dark:hover:bg-zinc-900 dark:border-zinc-300 dark:text-zinc-200 dark:hover:text-white "
      >
        {buttonDefault.avatar ? (
          <Image
            src={buttonDefault.avatar}
            height={20}
            width={20}
            title={buttonDefault.text}
            className=" rounded-full"
            alt={buttonDefault.text}
            priority
          />
        ) : undefined}
        {buttonDefault.text}
        <svg
          className="hs-dropdown-open:rotate-180 w-2.5 h-2.5 text-zinc-600 dark:text-zinc-200"
          width="16"
          height="16"
          viewBox="0 0 16 16"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
        >
          <path
            d="M2 5L8.16086 10.6869C8.35239 10.8637 8.64761 10.8637 8.83914 10.6869L15 5"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
          />
        </svg>
      </button>

      <div
        className="hs-dropdown-menu  w-72 transition-[opacity,margin] duration hs-dropdown-open:opacity-100 opacity-0 hidden z-10 duration-150 mt-2 min-w-[15rem] bg-white shadow-md rounded-lg p-2 dark:bg-zinc-800 dark:border dark:border-zinc-700 dark:divide-zinc-700"
        aria-labelledby="hs-dropdown-auto-close-outside"
      >
        {options.map((option) => {
          return (
            <button
              key={option.id}
              onClick={() => {
                SetButtonDefault(option)
                onClick(option.id)
              }}
              className="flex w-full items-center gap-x-3.5 py-2 px-3 rounded-md text-sm text-zinc-800 hover:bg-zinc-100 dark:text-zinc-400 dark:hover:bg-zinc-700 dark:hover:text-zinc-300"
            >
              {option.avatar ? (
                <Image
                  src={option.avatar}
                  height={26}
                  width={26}
                  title={option.text}
                  className=" rounded-full"
                  alt={option.text}
                  priority
                />
              ) : undefined}
              <span className="font-medium">{option.text}</span>
            </button>
          )
        })}
      </div>
    </div>
  )
}
