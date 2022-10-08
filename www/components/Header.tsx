import { useRouter } from "next/router";
import InternalLink from "./InternalLink";
import { SHUTTLE_DOCS_URL } from "../lib/constants";
import ExternalLink from "./ExternalLink";
import ThemeSwitch from "./ThemeSwitch";
import NoSsr from "./NoSsr";
import LoginButton from "./LoginButton";

const navigation = [
  { name: "Features", href: "/#features", internal: true },
  { name: "Examples", href: "/#examples", internal: true },
  { name: "Docs", href: SHUTTLE_DOCS_URL, internal: false },
  { name: "Blog", href: "/blog", internal: true },
  { name: "Pricing", href: "/pricing", internal: true },
];

export default function Header() {
  const { basePath } = useRouter();

  return (
    <header className="sticky top-0 z-20 bg-slate-100 !bg-opacity-70 dark:bg-dark-700">
      <nav className="mx-auto max-w-6xl px-4 sm:px-6 lg:px-8" aria-label="Top">
        <div className="flex w-full items-center justify-between py-3">
          <div className="flex items-center">
            <InternalLink href="/">
              <div className="relative m-auto flex">
                <img
                  className="h-8 w-auto"
                  src={`${basePath}/images/logo.png`}
                  alt="Shuttle"
                />
                <span className="absolute top-[-18px] right-[-19px] scale-[.45] rounded bg-brand-orange1 px-[10px] py-[2px] text-base font-bold text-slate-100 dark:text-dark-700">
                  ALPHA
                </span>
              </div>
            </InternalLink>
            <div className="ml-10 hidden space-x-8 lg:block">
              {navigation.map((link) =>
                link.internal ? (
                  <InternalLink
                    key={link.name}
                    href={link.href}
                    className="text-base font-medium text-slate-600 hover:text-slate-900 dark:text-gray-200 hover:dark:text-white"
                  >
                    {link.name}
                  </InternalLink>
                ) : (
                  <ExternalLink
                    key={link.name}
                    href={link.href}
                    className="text-base font-medium text-slate-600 hover:text-slate-900 dark:text-gray-200 hover:dark:text-white"
                  >
                    {link.name}
                  </ExternalLink>
                )
              )}
            </div>
          </div>
          <div className="ml-10 flex items-center space-x-4">
            <NoSsr>
              <ThemeSwitch />
            </NoSsr>

            <LoginButton />
          </div>
        </div>
        <div className="flex flex-wrap justify-center space-x-6 py-4 lg:hidden">
          {navigation.map((link) =>
            link.internal ? (
              <InternalLink
                key={link.name}
                href={link.href}
                className="text-base font-medium dark:text-gray-200 hover:dark:text-white"
              >
                {link.name}
              </InternalLink>
            ) : (
              <ExternalLink
                key={link.name}
                href={link.href}
                className="text-base font-medium dark:text-gray-200 hover:dark:text-white"
              >
                {link.name}
              </ExternalLink>
            )
          )}
        </div>
      </nav>
    </header>
  );
}
