
import { promises as fs } from "fs";
import { fileURLToPath } from 'url';
import path, { dirname } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

export async function getMappingFunc(mappingFile) {
    const mappingFilePath = path.join(__dirname, 'template/' + mappingFile);  // 模板文件路径
    const mappingFileStr = await fs.readFile(mappingFilePath, 'utf-8');
    const mapping = JSON.parse(mappingFileStr);

    return (key)=>{
        const val = mapping[key]
        if (val) {
            return val;
        }

        return key;
    }
}