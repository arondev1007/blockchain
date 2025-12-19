async function main(stream) {
	// 1) KV DB 리스트 불러오기
	const addressList = await qnLib.qnGetList("address");
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

	// 3) 트랜잭션 파싱
	for (const tx of block.transactions) {
		if (!tx.to) continue;

		// TO 주소 검증
		if (!targetAddresses.includes(tx.to.toLowerCase())) continue;
		filteredTxs.push(tx);
	}

	return {
		network: "ethereum",
		block: block_num,
		txs: filteredTxs,
	};
}

// 실행 - fn > base64
function encodeFilter(func) {
	const funcString = func.toString();
	const base64 = Buffer.from(funcString, "utf8").toString("base64");
	console.log(base64);
}

encodeFilter(main);
