import Big from "big.js";
import { BaseRPC } from "./base";
import * as ethers from "ethers";

export enum ChainID {
    main = 1,
    sepolia = 11155111,
}

interface RawTransaction {
    from: string;
    to: string;
    value: bigint;
    nonce: number;
    data?: string;
    gasLimit?: bigint;
    maxPriorityFeePerGas?: bigint;
    maxFeePerGas?: bigint;
    chainId: number;
}

export interface GasHistory {
    oldestBlock: string;
    reward: string[][];
    baseFeePerGas: string[];
    gasUsedRatio: number[];
    baseFeePerBlobGas?: string[]; // optional (EIP-4844)
    blobGasUsedRatio?: number[];
}

export interface Gas {
    maxPriorityFeePerGas: bigint;
    maxFeePerGas: bigint;
}

// https://www.quicknode.com/docs/ethereum/eth_sendRawTransaction
export class EthereumRPC extends BaseRPC {
    private chainId = ChainID.main;

    async getGas(): Promise<Gas> {
        const feeHistory = await this.getFeeHistory();

        // 다음 블록 기준 baseFee
        const lastBaseFee = feeHistory.baseFeePerGas.at(-1);
        if (lastBaseFee === undefined) {
            throw new Error("baseFeePerGas is empty");
        }

        const baseFeePerGas = BigInt(lastBaseFee);
        const priorityFeePerGas = feeHistory.reward?.[0]?.[0]
            ? BigInt(feeHistory.reward[0][0])
            : ethers.parseUnits("1.5", "gwei");

        const maxPriorityFeePerGas = priorityFeePerGas;
        const maxFeePerGas =
            baseFeePerGas * 2n + priorityFeePerGas;

        return {
            maxPriorityFeePerGas,
            maxFeePerGas,
        };
    }

    async getBlockHeight(): Promise<number> {
        const hex = await this.rpcRequest<string>("eth_blockNumber");
        return parseInt(hex, 16);
    }

    async getTransactionCount(address: string, blockNumber: string): Promise<number> {
        const nonce = await this.rpcRequest<string>(
            "eth_getTransactionCount",
            [address, blockNumber]
        );
        return parseInt(nonce, 16)
    }

    async getFeeHistory(): Promise<GasHistory> {
        const raw = await this.rpcRequest(
            "eth_feeHistory",
            [4, "latest", [25, 75]]
        );

        if (!this.isGasHistory(raw)) {
            throw new Error("Invalid eth_feeHistory response");
        }

        const gasHistory: GasHistory = {
            oldestBlock: raw.oldestBlock,
            reward: raw.reward,
            baseFeePerGas: raw.baseFeePerGas,
            gasUsedRatio: raw.gasUsedRatio,
            baseFeePerBlobGas: raw.baseFeePerBlobGas,
            blobGasUsedRatio: raw.blobGasUsedRatio,
        };

        return gasHistory;
    }

    async getBalance(address: string, blockNumber: string) {
        const balance = await this.rpcRequest<string>(
            "eth_getBalance",
            [address, blockNumber]
        );
        const wei = parseInt(balance, 16)
        return this.weiToEth(BigInt(wei));
    }

    async sendRawTransaction(signedTx: string): Promise<string> {
        return await this.rpcRequest(
            "eth_sendRawTransaction",
            [signedTx]
        );
    }

    async sign(privkey: string, rawTransaction: RawTransaction): Promise<string> {
        const wallet = new ethers.Wallet(privkey);
        const signedTx = await wallet.signTransaction(rawTransaction);
        return signedTx;
    }

    async setRawTransaction(
        addressFrom: string,
        addressTo: string,
        amount: string
    ): Promise<RawTransaction> {
        let big_gas_limit = 21000n;

        const gas = await this.getGas();
        const nonce = await this.getTransactionCount(addressFrom, "latest");
        const balance = await this.getBalance(addressFrom, "latest");
        const balanceEth = Big(balance);

        // 가스 비용 = 가스 한도 * 최대 가스 비용
        const big_gas_cost__wei = big_gas_limit * gas.maxFeePerGas;
        const gasCostWei = this.weiToEth(big_gas_cost__wei);

        // 최종 전송비용 = 송금량 - 가스비
        // ( 송금 수량 안에서 가스비를 뺀 수량을 전송 )
        const sendTotalEth = Big(amount).minus(gasCostWei);
        if (sendTotalEth.lt(0)) {
            throw new Error("err - insufficient amount to cover gas fee");
        }

        // 잔고 검증
        const sendTotalEthStr = sendTotalEth.toString();
        if (balanceEth.lt(amount)) {
            throw new Error(
                "err - insufficient balance" +
                "balance : " +
                balanceEth +
                " / send total : " +
                sendTotalEthStr
            );
        }

        const t_raw_tx: RawTransaction = {
            from: addressFrom,
            to: addressTo,
            value: ethers.parseEther(sendTotalEthStr),
            nonce: nonce,
            gasLimit: big_gas_limit,
            maxPriorityFeePerGas: gas.maxPriorityFeePerGas,
            maxFeePerGas: gas.maxFeePerGas,
            chainId: this.chainId,
        };

        return t_raw_tx;
    }

    weiToEth(_big_wei: bigint): string {
        const s_eth = ethers.formatEther(_big_wei);
        return s_eth;
    }

    ethToWei(_s_eth: string): bigint {
        const big_wei = ethers.parseEther(_s_eth);
        return big_wei;
    }

    private isGasHistory(data: any): data is GasHistory {
        return (
            typeof data === "object" &&
            data !== null &&
            typeof data.oldestBlock === "string" &&
            Array.isArray(data.baseFeePerGas) &&
            Array.isArray(data.gasUsedRatio) &&
            Array.isArray(data.reward)
        );
    }
}

async function main() {
    const eth = new EthereumRPC("https://snowy-burned-sound.quiknode.pro/8992b43cc48d6fe17a7b58b6a8498a3c5bf5cc12");

    // // 마지막 블록번호 조회
    // const block_num = await eth.getBlockHeight();
    // console.log("Block number:", block_num);

    // // 서명된 트랜잭션 전송
    // const txid = await eth.sendRawTransaction("");
    // console.log("txid:");

    // // 논스 조회
    // const nonce = await eth.getTransactionCount("0x9b4603F39520292403168cdB18786e575e5A120e", "latest");
    // console.log("nonce:", nonce)

    // // 잔고 조회
    // const balance = await eth.getBalance("0x9b4603F39520292403168cdB18786e575e5A120e", "latest");
    // console.log("balance:", balance)

    // // Fee 내역 조회
    // const gasHistroy = await eth.getFeeHistory();
    // console.log("gasHistroy:", gasHistroy)

    // // 가스 조회
    // const gas = await eth.getGas();
    // console.log("gas:", gas)

    // // 로우 트랜잭션 생성
    // const rawTx = await eth.setRawTransaction("0x9b4603F39520292403168cdB18786e575e5A120e", "0xe5d1Ec8198325cA39A60A38f0Aa3fA4bC3D91C46", "0.02");
    // console.log("rawTx: ", rawTx);

    // // 서명
    // const signedTx = await eth.sign("개인키", rawTx);
    // console.log("signedTx:", signedTx);

    // // 트랜잭션 전송
    // const txid = await eth.sendRawTransaction(signedTx);
    // console.log("txid:", txid);
}

main();