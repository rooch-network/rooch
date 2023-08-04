import { Command } from 'commander';
import { sh } from "./shell.js";
import { createTempFile, deleteTempFile } from "./utils.js"

const main = async (opts) => {
    const tmpFile = await createTempFile();
    const outputDir = opts.outputDir || "./src/generated/runtime"

    try {
        await sh("cargo", ["run", "--bin", "rooch", "abi", "export-rooch-types", "--file-path=" + tmpFile])
        await sh("cargo", ["install", "--version=0.1.0", "serde-generate-bin"])
        await sh("serdegen", ["--language=TypeScript", "--target-source-dir=" + outputDir, "--with-runtimes=Serde", tmpFile])
    } finally {
        await deleteTempFile(tmpFile)
    }
}

const program = new Command();
program
    .option('-o, --output-dir <string>', 'Output dir for generated typescript code.')
    .parse();

main(program.opts());
