import Image from "next/image";
import { usePathname } from "next/navigation";
import { getPagesUnderRoute } from "nextra/context";
import ROOCH_TEAM from "../../data/team";
import { useState, useEffect } from "react";

export default function PostHeader() {
  const pathname = usePathname();

  const [page, setPage] = useState(null);

  useEffect(() => {
    setPage(
      getPagesUnderRoute("/blog").find((page) => page.route === pathname)
    );
  }, [pathname]);

  return page ? (
    <div className="text-center inline-block mx-auto w-full">
      <h1 className="font-bold text-5xl mt-6">{page.frontMatter.title}</h1>
      <h2 className=" my-3 text-sm inline-flex gap-2 uppercase text-gray-500 dark:text-gray-300">
        {page.frontMatter.category}
        {" | "}
        <Image
          src={ROOCH_TEAM[page.frontMatter.author].avatar}
          alt={page.frontMatter.author}
          width={20}
          height={20}
          className="rounded-full"
        />
        <span className="font-semibold">
          <a
            href={
              "https://twitter.com/" +
              ROOCH_TEAM[page.frontMatter.author].twitterUsername
            }
            target="_blank"
          >
            {ROOCH_TEAM[page.frontMatter.author].name}
          </a>
        </span>
      </h2>
    </div>
  ) : undefined;
}
