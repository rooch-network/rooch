import { NextResponse } from 'next/server';
import fs from 'fs';
import path from 'path';

export async function GET(
  request: Request,
  { params }: { params: { directory: string } }
) {
  const { directory } = params;
  const readmePath = path.join(process.cwd(), '..', 'examples', directory, 'README.md');

  if (fs.existsSync(readmePath)) {
    const content = fs.readFileSync(readmePath, 'utf-8');
    return new NextResponse(content);
  } else {
    return new NextResponse('README.md not found', { status: 404 });
  }
}