import { BaseRPC } from "./base";

// https://www.quicknode.com/docs/bitcoin/getblock
export class BitcoinRPC extends BaseRPC {
    async getBlock(blockhash: string, verbosity = 1) {
        return this.rpcRequest(
            "getblock",
            [blockhash, verbosity]
        );
    }
}

async function main() {
    const btc = new BitcoinRPC("https://cold-still-ensemble.btc.quiknode.pro/5633a389a02abc86b8eb4a30edcbd892f0684839");
    const block = await btc.getBlock(
        "000000000000000000008be747f157ead9f3f175e5da3e6323020ddf716c45e3",
        1
    );

    console.log("Block data:", block);
}

main();