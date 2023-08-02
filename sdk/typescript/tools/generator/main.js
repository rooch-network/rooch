import { Generator } from "./generator.js"

const main = async () => {
    const generator = new Generator({
        openrpcDocument: "../../crates/rooch-open-rpc-spec/schemas/openrpc.json",
        outDir: "./src/generated/client"
    })

    try {
        await generator.execute()
        console.log("generate rooch typescript client ok!")
    } catch (e) {
        console.error("generate rooch typescript client error:", e)
    }
}

main();
