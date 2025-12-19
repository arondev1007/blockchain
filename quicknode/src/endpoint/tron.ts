import { Transaction, TransferContract } from "tronweb/lib/esm/types";
import { BaseRest, BaseRPC } from "./base";
import { BigNumber, TronWeb } from "tronweb";

export enum ChainID {
    main = "main",
    nile = "nile",
}

export enum ResourceType {
    bandwidth = 0,
    energy = 1,
}

// https://www.quicknode.com/docs/tron/eth_blockNumber
// Tron과 Eth 동일한 RPC 스팩을 이용 ( ContractTriger 부분 다름 )
export class TronRPC extends BaseRPC {
    async getBlockHeight(): Promise<number> {
        const hex = await this.rpcRequest<string>("eth_blockNumber");
        return parseInt(hex, 16);
    }
}

export class TronRest extends BaseRest {
    private tronWeb = new TronWeb({
        fullHost: "https://api.trongrid.io",
    });
    private chainID = ChainID.main;

    async broadcastTransaction(tx: any) {
        return this.post("/wallet/broadcasttransaction", tx);
    }
}


async function main() {
    const tron = new TronRPC("https://blissful-solemn-spree.tron-mainnet.quiknode.pro/c14ad9b6faa00b48b85693e5b43c32d83e705933/jsonrpc");

    // 마지막 블록번호 조회
    const block = await tron.getBlockHeight();
    console.log("Block number:", block);
}

main();