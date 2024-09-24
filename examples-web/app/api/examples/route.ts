import { NextResponse } from 'next/server';
import fs from 'fs';
import path from 'path';

function getFiles(dir: string, baseDir: string): any[] {
  const files = fs.readdirSync(dir, { withFileTypes: true });
  return files.map(file => {
    const filePath = path.join(dir, file.name);
    const relativePath = path.relative(baseDir, filePath);
    if (file.isDirectory()) {
      return {
        name: file.name,
        path: relativePath,
        type: 'directory',
        children: getFiles(filePath, baseDir)
      };
    } else {
      return {
        name: file.name,
        path: relativePath,
        type: 'file'
      };
    }
  });
}

export async function GET() {
  const examplesDir = path.join(process.cwd(), '..', 'examples');
  const directories = fs.readdirSync(examplesDir, { withFileTypes: true })
    .filter(dirent => dirent.isDirectory())
    .map(dirent => {
      const sourcesDir = path.join(examplesDir, dirent.name, 'sources');
      const files = fs.existsSync(sourcesDir) 
        ? getFiles(sourcesDir, sourcesDir)
        : [];
      return {
        name: dirent.name,
        files
      };
    });

  return NextResponse.json(directories);
}