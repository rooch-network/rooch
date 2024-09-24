"use client";

import { useState, useEffect, useRef } from "react";
import { marked } from "marked";
import hljs from "highlight.js/lib/core";
import "highlight.js/styles/github.css";

hljs.registerLanguage("move", function (hljs) {
  return {
    name: "Move",
    keywords: {
      keyword: "module struct public fun let mut return if else while loop",
      literal: "true false",
      built_in: "u8 u64 u128 address vector",
    },
    contains: [
      hljs.C_LINE_COMMENT_MODE,
      hljs.QUOTE_STRING_MODE,
      hljs.C_NUMBER_MODE,
    ],
  };
});

interface File {
  name: string;
  path: string;
  type: "file" | "directory";
  children?: File[];
}

interface Directory {
  name: string;
  files: File[];
}

export default function ExamplesPage() {
  const [directories, setDirectories] = useState<Directory[]>([]);
  const [selectedDirectory, setSelectedDirectory] = useState<string | null>(
    null
  );
  const [selectedFile, setSelectedFile] = useState<string | null>(null);
  const [content, setContent] = useState<string>("");
  const contentRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    fetch("/api/examples")
      .then((res) => res.json())
      .then((data) => setDirectories(data));
  }, []);

  useEffect(() => {
    if (selectedDirectory) {
      fetchReadme(selectedDirectory);
    }
  }, [selectedDirectory]);

  useEffect(() => {
    if (selectedDirectory && selectedFile) {
      fetchFileContent(selectedDirectory, selectedFile);
    }
  }, [selectedDirectory, selectedFile]);

  const fetchReadme = (directory: string) => {
    fetch(`/api/examples/${directory}/README.md`)
      .then((res) => {
        if (!res.ok) {
          throw new Error("README not found");
        }
        return res.text();
      })
      .then((text) => {
        const htmlContent = marked.parse(text);
        return htmlContent;
      })
      .then((htmlContent) => {
        setContent(htmlContent);
        scrollToTop();
      })
      .catch(() => {
        setContent(`<h1>${directory}</h1>`);
        scrollToTop();
      });
  };

  const fetchFileContent = (directory: string, filePath: string) => {
    fetch(`/api/examples/${directory}/sources/${encodeURIComponent(filePath)}`)
      .then((res) => res.text())
      .then((text) => {
        if (text.startsWith('<!DOCTYPE html>')) {
          setContent(`<h2 class="text-xl font-bold mb-4">Error loading file: ${filePath}</h2>`);
        } else {
          const highlightedCode = hljs.highlight(text, {
            language: "move",
          }).value;
          setContent(`
            <h2 class="text-xl font-bold mb-4">${filePath}</h2>
            <pre><code class="hljs language-move text-base">${highlightedCode}</code></pre>
          `);
        }
        scrollToTop();
      })
      .catch((error) => {
        setContent(`<h2 class="text-xl font-bold mb-4">Error loading file: ${filePath}</h2><p>${error.message}</p>`);
        scrollToTop();
      });
  };

  const scrollToTop = () => {
    if (contentRef.current) {
      contentRef.current.scrollTop = 0;
    }
  };

  const handleDirectoryClick = (dirName: string) => {
    setSelectedDirectory(dirName);
    setSelectedFile(null);
    fetchReadme(dirName);
  };

  const truncateName = (name: string, maxLength: number) => {
    if (name.length <= maxLength) return name;
    return name.slice(0, maxLength - 3) + "...";
  };

  const renderFileTree = (files: File[], depth: number = 0) => {
    return (
      <ul className={`ml-${depth * 4}`}>
        {files.map((file) => (
          <li key={file.path} className="mb-1">
            {file.type === "directory" ? (
              <div>
                <span className="font-bold">📁 {file.name}</span>
                {renderFileTree(file.children || [], depth + 1)}
              </div>
            ) : (
              <button
                className={`btn btn-xs btn-ghost btn-block justify-start ${
                  selectedFile === file.path ? "bg-base-300" : ""
                }`}
                onClick={() => {
                  setSelectedFile(file.path);
                  fetchFileContent(selectedDirectory!, file.path);
                }}
                title={file.name}
              >
                📄{" "}
                <span className="truncate">{truncateName(file.name, 23)}</span>
              </button>
            )}
          </li>
        ))}
      </ul>
    );
  };

  return (
    <div className="flex h-screen text-sm">
      <div className="w-64 bg-base-200 p-4 overflow-y-auto">
        <h2 className="text-lg font-bold mb-4">Examples</h2>
        <ul>
          {directories.map((dir) => (
            <li key={dir.name} className="mb-2">
              <button
                className={`btn btn-sm btn-ghost btn-block justify-start ${
                  selectedDirectory === dir.name ? "bg-base-300" : ""
                }`}
                onClick={() => handleDirectoryClick(dir.name)}
                title={dir.name}
              >
                📁{" "}
                <span className="truncate">{truncateName(dir.name, 23)}</span>
              </button>
              <div
                className={`mt-2 ml-4 overflow-hidden transition-all duration-300 ease-in-out ${
                  selectedDirectory === dir.name ? "max-h-[500px]" : "max-h-0"
                }`}
              >
                {renderFileTree(dir.files)}
              </div>
            </li>
          ))}
        </ul>
      </div>
      <div ref={contentRef} className="flex-1 p-4 overflow-y-auto">
        <div
          dangerouslySetInnerHTML={{ __html: content }}
          className="prose prose-sm max-w-none [&_pre]:text-base"
        />
      </div>
    </div>
  );
}
