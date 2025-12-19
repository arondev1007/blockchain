export abstract class BaseRPC {
    protected endpoint: string;

    constructor(endpoint: string) {
        this.endpoint = endpoint.replace(/\/+$/, "");
    }

    protected headers() {
        return {
            "Content-Type": "application/json",
            Accept: "application/json",
        };
    }

    protected async rpcRequest<T>(
        method: string,
        params: any[] = [],
        id = 1
    ): Promise<T> {
        const res = await fetch(this.endpoint, {
            method: "POST",
            headers: this.headers(),
            body: JSON.stringify({
                jsonrpc: "2.0",
                id,
                method,
                params,
            }),
        });

        if (!res.ok) {
            throw new Error(await res.text());
        }

        const data = await res.json();

        if (data.error) {
            throw new Error(JSON.stringify(data.error));
        }

        return data.result;
    }
}

export abstract class BaseRest {
    protected endpoint: string;

    constructor(endpoint: string) {
        this.endpoint = endpoint.replace(/\/+$/, "");
    }

    protected async post<T>(
        path: string,
        body: any
    ): Promise<T> {
        const res = await fetch(`${this.endpoint}${path}`, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(body),
        });

        if (!res.ok) {
            throw new Error(await res.text());
        }

        return await res.json() as T;
    }
}