import axios from "axios";

export enum Network {
	ethereum = "ethereum-mainnet",
	bitcoin = "bitcoin-mainnet",
	tron = "tron-mainnet",
}

export enum Compression {
	none = "none",
	gzip = "gzip",
}

export enum IncludeMetaData {
	body = "body",
	header = "header",
	none = "none",
}

export interface WebhookDestinationAttributes {
	url: string;
	compression: Compression;
	headers: Record<string, string>;
	max_retry: number;
	retry_interval_sec: number;
	post_timeout_sec: number;
}

// 일단 웹훅만 지원
export enum Destination {
	webhook = "webhook",
	s3 = "s3",
	azure = "azure",
	postgres = "postgres",
}

export enum StreamStatus {
	active = "active",
	paused = "paused",
}

// 현재 단일 리전만 지원
export enum Region {
	usa_east = "usa_east",
}

export interface CreateStreamArgs {
	// REQUIRED
	name: string;
	network: Network;
	include_stream_metadata: IncludeMetaData;
	elastic_batch_enabled: boolean;
	status: StreamStatus;
	destination: Destination;
	destination_attributes: WebhookDestinationAttributes;
	dataset_batch_size: number;

	// OPTIONAL
	region?: Region;
	start_range?: number;
	end_range?: number;
	fix_block_reorgs?: number;
	keep_distance_from_tip?: number;
	notification_email?: string;
}

export interface UpdateStreamArgs {
	// REQUIRED
	name: string;
	include_stream_metadata: IncludeMetaData;
	dataset_batch_size: number;
	destination: Destination;
	destination_attributes: WebhookDestinationAttributes;
	status: StreamStatus;

	// OPTIONAL
	filter_function?: string; // base64-encoded JS
	start_range?: number;
	end_range?: number;

	region?: "usa_east";
	elastic_batch_enabled?: boolean;
	fix_block_reorgs?: number;
	keep_distance_from_tip?: number;
	notification_email?: string;
}

export class Stream {
	private apiKey: string;
	private apiUrl: string;

	constructor(apiKey: string) {
		this.apiKey = apiKey;
		this.apiUrl = "https://api.quicknode.com/streams/rest/v1/streams";
	}

	private get headers() {
		return {
			accept: "application/json",
			"Content-Type": "application/json",
			"x-api-key": this.apiKey,
		};
	}

	// Fn -> Base64
	encodeFilter(func: Function): string {
		if (typeof func !== "function") {
			throw new Error("encodeFunctionToBase64: Provided value is not a function");
		}

		const funcString = func.toString();
		return Buffer.from(funcString, "utf8").toString("base64");
	}

	async create(args: CreateStreamArgs) {
		const { data_set, filter_fn } = this.loadFilterFn(args.network);

		const body: any = {
			name: args.name,
			network: args.network,
			dataset: data_set,
			filter_function: filter_fn,

			include_stream_metadata: args.include_stream_metadata,
			elastic_batch_enabled: args.elastic_batch_enabled,

			region: args.region ?? Region.usa_east,
			dataset_batch_size: args.dataset_batch_size ?? 1,

			fix_block_reorgs: args.fix_block_reorgs ?? 0,
			keep_distance_from_tip: args.keep_distance_from_tip ?? 0,

			destination: args.destination,
			destination_attributes: args.destination_attributes,
			status: args.status ?? StreamStatus.active,
		};

		// start_range / end_range
		if (args.start_range !== undefined) body.start_range = args.start_range;
		if (args.end_range !== undefined) body.end_range = args.end_range;

		const res = await axios.post(this.apiUrl, body, {
			headers: this.headers,
		});

		return res.data;
	}

	async get(streamId: string) {
		const res = await axios.get(`${this.apiUrl}/${streamId}`, {
			headers: this.headers,
		});
		return res.data;
	}

	async update(streamId: string, opts: UpdateStreamArgs) {
		const body: any = {};

		// name
		if (opts.name !== undefined) body.name = opts.name;

		// filter_function
		if (opts.filter_function !== undefined) {
			body.filter_function = opts.filter_function;
		}

		// include_stream_metadata
		if (opts.include_stream_metadata !== undefined) {
			body.include_stream_metadata = opts.include_stream_metadata;
		}

		// start_range / end_range
		if (opts.start_range !== undefined) body.start_range = opts.start_range;
		if (opts.end_range !== undefined) body.end_range = opts.end_range;

		// dataset_batch_size
		if (opts.dataset_batch_size !== undefined) {
			body.dataset_batch_size = opts.dataset_batch_size;
		}

		// fix_block_reorgs / keep_distance_from_tip
		if (opts.fix_block_reorgs !== undefined) body.fix_block_reorgs = opts.fix_block_reorgs;
		if (opts.keep_distance_from_tip !== undefined)
			body.keep_distance_from_tip = opts.keep_distance_from_tip;

		// destination
		if (opts.destination !== undefined) body.destination = opts.destination;

		// destination_attributes
		if (opts.destination_attributes !== undefined)
			body.destination_attributes = opts.destination_attributes;

		// status
		if (opts.status !== undefined) body.status = opts.status;

		const res = await axios.patch(`${this.apiUrl}/${streamId}`, body, {
			headers: this.headers,
		});

		return res.data;
	}

	async delete(streamId: string) {
		const res = await axios.delete(`${this.apiUrl}/${streamId}`, {
			headers: this.headers,
		});
		return res.data;
	}

	private loadFilterFn(network: Network) {
		let data_set = "";
		let filter_fn = "";
		switch (network) {
			case Network.ethereum:
				data_set = "block_with_receipts";
				filter_fn =
					"YXN5bmMgZnVuY3Rpb24gbWFpbihzdHJlYW0pIHsNCgkvLyAxKSBLViBEQiDrpqzsiqTtirgg67aI65+s7Jik6riwDQoJY29uc3QgYWRkcmVzc0xpc3QgPSBhd2FpdCBxbkxpYi5xbkdldExpc3QoImFkZHJlc3MiKTsNCglpZiAoIUFycmF5LmlzQXJyYXkoYWRkcmVzc0xpc3QpKSB7DQoJCXJldHVybiB7IGVycm9yOiAiYWRkcmVzcyDrpqzsiqTtirjrpbwg67aI65+s7Jik7KeAIOuqu+2WiOyKteuLiOuLpC4iIH07DQoJfQ0KDQoJLy8g7IaM66y47J6QIOuzgOqyvQ0KCWNvbnN0IHRhcmdldEFkZHJlc3NlcyA9IGFkZHJlc3NMaXN0Lm1hcCgoYSkgPT4gYS50b0xvd2VyQ2FzZSgpKTsNCg0KCS8vIDIpIOu4lOuhnSDtjIzsi7ENCgljb25zdCBibG9jayA9IHN0cmVhbS5kYXRhPy5bMF0/LmJsb2NrOw0KCWlmICghYmxvY2spIHJldHVybiB7IGJsb2NrTnVtYmVyOiBudWxsIH07DQoNCgljb25zdCBibG9ja19udW0gPSBwYXJzZUludChibG9jay5udW1iZXIsIDE2KTsNCgljb25zdCBmaWx0ZXJlZFR4cyA9IFtdOw0KDQoJLy8gMykg7Yq4656c7J6t7IWYIO2MjOyLsQ0KCWZvciAoY29uc3QgdHggb2YgYmxvY2sudHJhbnNhY3Rpb25zKSB7DQoJCWlmICghdHgudG8pIGNvbnRpbnVlOw0KDQoJCS8vIFRPIOyjvOyGjCDqsoDspp0NCgkJaWYgKCF0YXJnZXRBZGRyZXNzZXMuaW5jbHVkZXModHgudG8udG9Mb3dlckNhc2UoKSkpIGNvbnRpbnVlOw0KCQlmaWx0ZXJlZFR4cy5wdXNoKHR4KTsNCgl9DQoNCglyZXR1cm4gew0KCQluZXR3b3JrOiAiZXRoZXJldW0iLA0KCQlibG9jazogYmxvY2tfbnVtLA0KCQl0eHM6IGZpbHRlcmVkVHhzLA0KCX07DQp9";
				break;

			case Network.bitcoin:
				data_set = "block";
				filter_fn =
					"YXN5bmMgZnVuY3Rpb24gbWFpbihwYXlsb2FkKSB7DQoJY29uc3QgZGF0YSA9IHBheWxvYWQuZGF0YTsNCgljb25zdCB0eHMgPSBkYXRhPy5bMF0/LnR4czsNCglpZiAoIXR4cykgcmV0dXJuIHBheWxvYWQ7DQoNCgkvLyAxKSBLViBEQiDrpqzsiqTtirgg67aI65+s7Jik6riwDQoJY29uc3QgYWRkcmVzc0xpc3QgPSBhd2FpdCBxbkxpYi5xbkdldExpc3QoImFkZHJfYnRjIik7DQoJaWYgKCFBcnJheS5pc0FycmF5KGFkZHJlc3NMaXN0KSkgew0KCQlyZXR1cm4geyBlcnJvcjogImFkZHJlc3Mg66as7Iqk7Yq466W8IOu2iOufrOyYpOyngCDrqrvtlojsirXri4jri6QuIiB9Ow0KCX0NCg0KCS8vIDIpIOu4lOuhnSDtjIzsi7ENCgljb25zdCBibG9jayA9IGRhdGE/LlswXT8uaGVpZ2h0Ow0KCWlmICghYmxvY2spIHJldHVybiB7IGJsb2NrTnVtYmVyOiBudWxsIH07DQoJY29uc3QgYmxvY2tfbnVtID0gcGFyc2VJbnQoYmxvY2subnVtYmVyLCAxNik7DQoNCgkvLyDshozrrLjsnpAg67OA6rK9DQoJY29uc3QgdGFyZ2V0QWRkcmVzc2VzID0gYWRkcmVzc0xpc3QubWFwKChhKSA9PiBhLnRvTG93ZXJDYXNlKCkpOw0KCWNvbnN0IGZpbHRlcmVkVHhzID0gW107DQoNCgkvLyDtirjrnpzsnq3shZgg7YyM7IuxDQoJZm9yIChjb25zdCB0eCBvZiB0eHMpIHsNCgkJZm9yIChjb25zdCB2IG9mIHR4LnZvdXQpIHsNCgkJCWlmICh2LmlzQWRkcmVzcyA9PT0gZmFsc2UpIHsNCgkJCQljb250aW51ZTsNCgkJCX0NCg0KCQkJLy8gVE8g7KO87IaMIOqygOymnQ0KCQkJZm9yIChjb25zdCBhZGRyZXNzIG9mIHYuYWRkcmVzc2VzKSB7DQoJCQkJaWYgKCF0YXJnZXRBZGRyZXNzZXMuaW5jbHVkZXMoYWRkcmVzcy50b0xvd2VyQ2FzZSgpKSkgY29udGludWU7DQoJCQkJZmlsdGVyZWRUeHMucHVzaCh0eCk7DQoJCQl9DQoNCgkJCWNvbnNvbGUubG9nKHYuYWRkcmVzc2VzKTsNCgkJfQ0KCX0NCg0KCXJldHVybiB7DQoJCW5ldHdvcms6ICJiaXRjb2luIiwNCgkJYmxvY2s6IGJsb2NrX251bSwNCgkJdHhzOiBmaWx0ZXJlZFR4cywNCgl9Ow0KfQ==";
				break;

			case Network.tron:
				data_set = "block_with_receipts";
				filter_fn =
					"YXN5bmMgZnVuY3Rpb24gbWFpbihzdHJlYW0pIHsNCgkvLyAxKSBLViBEQiDrpqzsiqTtirgg67aI65+s7Jik6riwDQoJY29uc3QgYWRkcmVzc0xpc3QgPSBhd2FpdCBxbkxpYi5xbkdldExpc3QoImFkZHJfdHJvbiIpOw0KCWlmICghQXJyYXkuaXNBcnJheShhZGRyZXNzTGlzdCkpIHsNCgkJcmV0dXJuIHsgZXJyb3I6ICJhZGRyZXNzIOumrOyKpO2KuOulvCDrtojrn6zsmKTsp4Ag66q77ZaI7Iq164uI64ukLiIgfTsNCgl9DQoNCgkvLyDshozrrLjsnpAg67OA6rK9DQoJY29uc3QgdGFyZ2V0QWRkcmVzc2VzID0gYWRkcmVzc0xpc3QubWFwKChhKSA9PiBhLnRvTG93ZXJDYXNlKCkpOw0KDQoJLy8gMikg67iU66GdIO2MjOyLsQ0KCWNvbnN0IGJsb2NrID0gc3RyZWFtLmRhdGE/LlswXT8uYmxvY2s7DQoJaWYgKCFibG9jaykgcmV0dXJuIHsgYmxvY2tOdW1iZXI6IG51bGwgfTsNCg0KCWNvbnN0IGJsb2NrX251bSA9IHBhcnNlSW50KGJsb2NrLm51bWJlciwgMTYpOw0KCWNvbnN0IGZpbHRlcmVkVHhzID0gW107DQoJY29uc3QgZmlsdGVyZWRSZWNlaXB0cyA9IFtdOw0KDQoJLy8gMykg7Yq4656c7J6t7IWYIO2MjOyLsQ0KCWZvciAoY29uc3QgdHggb2YgYmxvY2sudHJhbnNhY3Rpb25zKSB7DQoJCWlmICghdHgudG8pIGNvbnRpbnVlOw0KDQoJCS8vIFRPIOyjvOyGjCDqsoDspp0NCgkJaWYgKCF0YXJnZXRBZGRyZXNzZXMuaW5jbHVkZXModHgudG8udG9Mb3dlckNhc2UoKSkpIGNvbnRpbnVlOw0KCQlmaWx0ZXJlZFR4cy5wdXNoKHR4KTsNCgl9DQoNCgkvLyA0KSDsmIHsiJjspp0g7YyM7IuxDQoJY29uc3QgcmVjZWlwdHMgPSBzdHJlYW0uZGF0YT8uWzBdPy5yZWNlaXB0czsNCglmb3IgKGNvbnN0IHJlY2VpcHQgb2YgcmVjZWlwdHMpIHsNCgkJZm9yIChjb25zdCBsb2cgb2YgcmVjZWlwdC5sb2dzKSB7DQoJCQkvLyBjaGVjayAtIGNvbnRyYWN0IGFkZHJlc3MgKHVzZHQpDQoJCQlpZiAoDQoJCQkJbG9nLmFkZHJlc3MgIT09ICIweDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMGE2MTRmODAzYjZmZDc4MDk4NmE0MmM3OGVjOWM3Zjc3ZTZkZWQxM2MiDQoJCQkpDQoJCQkJY29udGludWU7DQoNCgkJCS8vIGNoZWNrIC0gZXZlbnQgdHlwZSAoVHJhbnNmZXIpDQoJCQlpZiAoDQoJCQkJbG9nLnRvcGljcz8uWzBdICE9PQ0KCQkJCSIweGRkZjI1MmFkMWJlMmM4OWI2OWMyYjA2OGZjMzc4ZGFhOTUyYmE3ZjE2M2M0YTExNjI4ZjU1YTRkZjUyM2IzZWYiDQoJCQkpDQoJCQkJY29udGludWU7DQoNCgkJCS8vIGNoZWNrIC0gYWRkcmVzcyB0bw0KCQkJY29uc3QgYWRkciA9ICIweCIgKyBsb2cudG9waWNzPy5bMl0uc2xpY2UoMjYpOw0KCQkJY29uc29sZS5sb2coYWRkcik7DQoNCgkJCS8vIFRPIOyjvOyGjCDqsoDspp0NCgkJCWlmICghdGFyZ2V0QWRkcmVzc2VzLmluY2x1ZGVzKGFkZHIpKSBjb250aW51ZTsNCgkJCWZpbHRlcmVkUmVjZWlwdHMucHVzaChsb2cpOw0KCQl9DQoJfQ0KDQoJcmV0dXJuIHsNCgkJbmV0d29yazogInRyb24iLA0KCQlibG9jazogYmxvY2tfbnVtLA0KCQl0eHM6IGZpbHRlcmVkVHhzLA0KCQlyZWNlaXB0czogZmlsdGVyZWRSZWNlaXB0cywNCgl9Ow0KfQ==";
				break;
		}
		return {
			data_set,
			filter_fn,
		};
	}
}

async function test() {
	const stream = new Stream("QN_5dd253b06c1245b19de6d67f9b05f650");

	// const result = await stream.create({
	// 	name: "eth-test",
	// 	network: Network.ethereum,
	// 	include_stream_metadata: IncludeMetaData.body,
	// 	elastic_batch_enabled: true,
	// 	status: StreamStatus.active,
	// 	dataset_batch_size: 1,
	// 	fix_block_reorgs: 0,
	// 	keep_distance_from_tip: 0,
	// 	destination: Destination.webhook,
	// 	destination_attributes: {
	// 		url: "https://webhook.site/2b0069c6-a99c-4d82-8d70-a590d779fc5f",
	// 		compression: Compression.none,
	// 		headers: {
	// 			"Content-Type": "application/json",
	// 		},
	// 		max_retry: 3,
	// 		retry_interval_sec: 1,
	// 		post_timeout_sec: 10,
	// 	},
	// });
	// console.log("Stream Created:", result);

	// ----------------------------------------------------------------------------------------
	// UPDATE STREAM (예시)
	// const update_result = await streams.update("12c64ece-13b6-43e4-a58e-a2c3c6e2208b", {
	// 	name: "eth-test",
	// 	include_stream_metadata: IncludeMetaData.body,
	// 	destination: Destination.webhook,

	// 	dataset_batch_size: 1,
	// 	fix_block_reorgs: 0,
	// 	keep_distance_from_tip: 0,

	// 	destination_attributes: {
	// 		url: "https://webhook.site",
	// 		compression: Compression.none,
	// 		headers: {
	// 			"Content-Type": "application/json",
	// 			Authorization: "again",
	// 		},
	// 		max_retry: 3,
	// 		retry_interval_sec: 1,
	// 		post_timeout_sec: 10,
	// 	},

	// 	status: StreamStatus.paused,
	// });
	// console.log("Updated:", update_result);

	// ----------------------------------------------------------------------------------------
	// DELETE STREAM
	// const delete_result = await streams.delete("12c64ece-13b6-43e4-a58e-a2c3c6e2208b");
	// console.log("Deleted:", delete_result);
}

test();
