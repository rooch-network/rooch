import { NextResponse } from 'next/server';
import fs from 'fs';
import path from 'path';

export async function GET(
  request: Request,
  { params }: { params: { directory: string; file: string } }
) {
  const { directory, file } = params;
  const decodedFile = decodeURIComponent(file);
  const filePath = path.join(process.cwd(), '..', 'examples', directory, 'sources', decodedFile);

  if (fs.existsSync(filePath)) {
    const content = fs.readFileSync(filePath, 'utf-8');
    return new NextResponse(content);
  } else {
    return new NextResponse('File not found', { status: 404 });
  }
}