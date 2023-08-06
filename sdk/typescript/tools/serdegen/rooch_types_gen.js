import { Command } from 'commander';
import { sh } from "./shell.js";
import { createTempFile, deleteTempFile, replaceFile} from "./utils.js"

const main = async (opts) => {
    const tmpFile = await createTempFile("rooch_types.yml");
    const outputDir = opts.outputDir || "./src/generated/runtime"

    try {
        await sh("cargo", ["run", "--bin", "rooch", "abi", "export-rooch-types", "--file-path=" + tmpFile])
        await sh("cargo", ["install", "--version=0.1.0", "serde-generate-bin"])
        await sh("serdegen", ["--language=TypeScript", "--target-source-dir=" + outputDir, "--with-runtimes=Bcs", tmpFile])
        await sh("serdegen", ["--language=TypeScript", "--target-source-dir=" + outputDir, "--with-runtimes=Serde", tmpFile])
        await replaceFile(outputDir + "/serde/binaryDeserializer.ts", "https://deno.land/std@0.85.0/node/util.ts", "@kayahr/text-encoding")
        await replaceFile(outputDir + "/serde/binarySerializer.ts", "https://deno.land/std@0.85.0/node/util.ts", "@kayahr/text-encoding")
    } finally {
        await deleteTempFile(tmpFile)
    }
}

const program = new Command();
program
    .option('-o, --output-dir <string>', 'Output dir for generated typescript code.')
    .parse();

main(program.opts());
