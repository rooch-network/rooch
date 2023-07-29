import { promises as fs} from "fs";
import { Generator } from "./generator.js"

const main = async ()=>{
    const roochOpenRPCText = await fs.readFile("../../crates/rooch-open-rpc-spec/schemas/openrpc.json", "utf-8");
    const roochOpenRPC = JSON.parse(roochOpenRPCText);

    try {
        const generator = new Generator("typescript", "./src/client")
        await generator.execute(roochOpenRPC)

        console.log("gen ok!")
    } catch(e) {
        console.log("gen error:", e)
    }
}

main();