import "./globals.css";
import "highlight.js/styles/github.css";
import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "Examples Page",
  description: "Browse through examples and their source code",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" data-theme="light">
      <body>{children}</body>
    </html>
  );
}
