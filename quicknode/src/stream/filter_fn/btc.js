async function main(payload) {
	const data = payload.data;
	const txs = data?.[0]?.txs;
	if (!txs) return payload;

	// 1) KV DB 리스트 불러오기
	const addressList = await qnLib.qnGetList("addr_btc");
	if (!Array.isArray(addressList)) {
		return { error: "address 리스트를 불러오지 못했습니다." };
	}

	// 2) 블록 파싱
	const block = data?.[0]?.height;
	if (!block) return { blockNumber: null };
	const block_num = parseInt(block.number, 16);

	// 소문자 변경
	const targetAddresses = addressList.map((a) => a.toLowerCase());
	const filteredTxs = [];

	// 트랜잭션 파싱
	for (const tx of txs) {
		for (const v of tx.vout) {
			if (v.isAddress === false) {
				continue;
			}

			// TO 주소 검증
			for (const address of v.addresses) {
				if (!targetAddresses.includes(address.toLowerCase())) continue;
				filteredTxs.push(tx);
			}

			console.log(v.addresses);
		}
	}

	return {
		network: "bitcoin",
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
