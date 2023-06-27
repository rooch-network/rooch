import { getPagesUnderRoute,} from "nextra/context";
import Link from "next/link";
import { useState, useEffect } from "react";
import { useRouter } from "next/router";
import { FilterButton } from "./filterButton";
import ROOCH_TEAM from "../../data/team";
import Image from "next/image";

export default function BlogIndex({
  textAllCategories = "All Categories",
  textAllAuthors = "All Authors",
  textMore = "Read more",
}) {
  const { locale } = useRouter();
  const [selectedCategory, setSelectedCategory] = useState(textAllCategories);
  const [selectedAuthor, setSelectedAuthor] = useState(textAllAuthors);

  const rawPages = getPagesUnderRoute("/blog");
  const [pages, SetPages] = useState(rawPages);
  const [pagesFiltered, setPagesFiltered] = useState(pages);

  // get all the authors
  const [authors, __] = useState(() => {
    let _authors = [textAllAuthors];

    pages.forEach((page) => {
      _authors = _authors.concat(page.frontMatter.author);
    });

    return Array.from(new Set(_authors));
  });

  // get all the categories
  const [categories, ___] = useState(() => {
    let _categories = [textAllCategories];

    pages.forEach((page) => {
      if (page.frontMatter.category) {
        _categories = _categories.concat(page.frontMatter.category);
      }
    });

    return Array.from(new Set(_categories));
  });

  // process date
  useEffect(() => {
    let _pages = [];
    _pages = pages.map((page: any) => {
      let _page = page;

      const options: Intl.DateTimeFormatOptions = {
        year: "numeric",
        month: "long",
        day: "numeric",
      };
      let dateObject = new Date(page.frontMatter?.date);
      _page.dateNumber = dateObject.getTime();

      const formmatedDate = dateObject.toLocaleDateString(page.locale, options);

      _page.frontMatter.date =
        formmatedDate != "Invalid Date"
          ? formmatedDate
          : _page.frontMatter.date;

      return _page;
    });
    SetPages(_pages);
  }, []);

  // filter pages
  useEffect(() => {
    let _pages = [];

    // filter based on locale
    _pages = pages.filter((page: any) => page.locale === locale);

    // filter based on selected author and category
    if (
      selectedAuthor === textAllAuthors &&
      selectedCategory == textAllCategories
    ) {
      _pages = _pages;
    } else {
      _pages = _pages.filter((page) => {
        const _author = String(page.frontMatter.author);
        const _category = String(page.frontMatter.category);
        return (
          (_author == selectedAuthor && _category == selectedCategory) ||
          (_author == selectedAuthor &&
            selectedCategory == textAllCategories) ||
          (selectedAuthor == textAllAuthors && _category == selectedCategory)
        );
      });
    }
    setPagesFiltered(_pages);
  }, [selectedCategory, selectedAuthor]);

  return (
    <div className="mt-10">
      <div className="flex gap-4 pb-2 mb-6">
        <FilterButton
          options={categories.map((category) => ({
            id: category,
            text: category,
            avatar: undefined,
          }))}
          onClick={(tag) => {
            setSelectedCategory(tag);
          }}
        />
        <FilterButton
          options={authors.map((author) => ({
            id: author,
            text: ROOCH_TEAM[author] ? ROOCH_TEAM[author].name : author,
            avatar: ROOCH_TEAM[author] ? ROOCH_TEAM[author].avatar : undefined,
          }))}
          onClick={(author) => {
            setSelectedAuthor(author);
          }}
        />
      </div>

      {pagesFiltered
        .sort((p1, p2) =>
          p1.dateNumber < p2.dateNumber
            ? 1
            : p1.dateNumber > p2.dateNumber
              ? -1
              : 0
        )
        .map((page) => {
          return (
            <div key={page.route}>
              <Link href={page.route}>
                <button className="mb-10 w-full text-left postbox focus:bg-gray-100 dark:focus:bg-gray-800 pl-4 py-6 rounded-2xl ">
                  {/* Post Category */}
                  <p className="-mb-1 text-sm uppercase inline-block text-gray-500 ">
                    {page.frontMatter.category}
                  </p>

                  {/* Post Title */}
                  <h3 className="block font-semibold text-2xl">
                    {page.meta?.title || page.frontMatter.title || page.name}
                  </h3>

                  {/* Post Author */}
                  <p className="inline-flex gap-2 mt-5">
                    {page.frontMatter.author ? (
                      <Image
                        src={String(ROOCH_TEAM[page.frontMatter.author].avatar)}
                        width={32}
                        height={32}
                        alt={page.frontMatter.author}
                        className="rounded-full"
                      />
                    ) : undefined}
                    {ROOCH_TEAM[page.frontMatter.author] ? (
                      <span className="font-medium text-xl">
                        {ROOCH_TEAM[page.frontMatter.author].name}
                      </span>
                    ) : (
                      page.frontMatter.author
                    )}
                  </p>

                  {/* Post Description */}
                  {page.frontMatter.description ? (
                    <p className="opacity-80 mt-2 leading-7">
                      {page.frontMatter.description}
                    </p>
                  ) : null
                  }

                  {/* Post Date */}
                  {page.frontMatter.date ? (
                    <p className="opacity-50 text-sm leading-7">
                      {page.frontMatter.date}
                    </p>
                  ) : null}
                </button>
              </Link>
            </div>
          );
        })}
    </div>
  );
}
