async function main(stream) {
	// 1) KV DB 리스트 불러오기
	const addressList = await qnLib.qnGetList("addr_tron");
	if (!Array.isArray(addressList)) {
		return { error: "address 리스트를 불러오지 못했습니다." };
	}

	// 소문자 변경
	const targetAddresses = addressList.map((a) => a.toLowerCase());

	// 2) 블록 파싱
	const block = stream.data?.[0]?.block;
	if (!block) return { blockNumber: null };

	const block_num = parseInt(block.number, 16);
	const filteredTxs = [];
	const filteredReceipts = [];

	// 3) 트랜잭션 파싱
	for (const tx of block.transactions) {
		if (!tx.to) continue;

		// TO 주소 검증
		if (!targetAddresses.includes(tx.to.toLowerCase())) continue;
		filteredTxs.push(tx);
	}

	// 4) 영수증 파싱
	const receipts = stream.data?.[0]?.receipts;
	for (const receipt of receipts) {
		for (const log of receipt.logs) {
			// check - contract address (usdt)
			if (
				log.address !== "0x000000000000000000000000a614f803b6fd780986a42c78ec9c7f77e6ded13c"
			)
				continue;

			// check - event type (Transfer)
			if (
				log.topics?.[0] !==
				"0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
			)
				continue;

			// check - address to
			const addr = "0x" + log.topics?.[2].slice(26);
			console.log(addr);

			// TO 주소 검증
			if (!targetAddresses.includes(addr)) continue;
			filteredReceipts.push(log);
		}
	}

	return {
		network: "tron",
		block: block_num,
		txs: filteredTxs,
		receipts: filteredReceipts,
	};
}

// 실행 - fn > base64
function encodeFilter(func) {
	const funcString = func.toString();
	const base64 = Buffer.from(funcString, "utf8").toString("base64");
	console.log(base64);
}

encodeFilter(main);
