import Link from "next/link";

export default function TitleHero({
  title,
  slogan,
  description,
  button,
  buttonHref,
}) {
  return (
    <div className="mx-auto px-4 sm:px-6 lg:px-8 pt-60 pb-60">
      {/* <!-- Title --> */}
      <div className="mt-5 max-w-3xl text-center mx-auto">
        <p className="block font-bold text-gray-800 text-4xl md:text-5xl lg:text-6xl dark:text-gray-200">
          {title}
        </p>
        <p className="block pt-2 font-bold text-gray-800 text-2xl md:text-3xl lg:text-4xl dark:text-gray-200">
          {slogan}
        </p>
      </div>
      {/* <!-- End Title --> */}

      <div className="mt-5 max-w-2xl text-center mx-auto">
        <p className="text-lg text-gray-600 dark:text-gray-400">
          {description}
        </p>
      </div>

      {/* <!-- Buttons --> */}
      <div className="mt-8 grid gap-3 w-full sm:inline-flex sm:justify-center cta-container">
        <Link
          className="inline-flex justify-center items-center gap-x-3 text-center bg-gradient-to-tl border border-transparent text-sm font-medium rounded-md  py-3 px-6 cta"
          href={buttonHref}
        >
          {button}
          <svg
            className="w-3 h-3"
            width="16"
            height="16"
            viewBox="0 0 16 16"
            fill="none"
          >
            <path
              d="M5.27921 2L10.9257 7.64645C11.1209 7.84171 11.1209 8.15829 10.9257 8.35355L5.27921 14"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
            />
          </svg>
        </Link>
      </div>
      {/* <!-- End Buttons --> */}
    </div>

    // <!-- End Hero -->
  );
}
