import { ProtocolWrapper, WSDOMClient, WSDOMTransport } from "./transport.js";

function assert(condition: unknown, message: string): asserts condition {
	if (!condition) throw new Error(message);
}

function equal<T>(actual: T, expected: T, message: string): void {
	assert(Object.is(actual, expected), `${message}: expected ${String(expected)}, received ${String(actual)}`);
}

async function tick(): Promise<void> {
	await Promise.resolve();
	await Promise.resolve();
}

class FakeWebSocket {
	static readonly CONNECTING = 0;
	static readonly OPEN = 1;
	static readonly CLOSING = 2;
	static readonly CLOSED = 3;
	static instances: FakeWebSocket[] = [];
	readyState = FakeWebSocket.CONNECTING;
	sent: string[] = [];
	onopen: ((event: Event) => void) | null = null;
	onmessage: ((event: MessageEvent) => void) | null = null;
	onclose: ((event: CloseEvent) => void) | null = null;
	onerror: ((event: Event) => void) | null = null;

	constructor(readonly url: string | URL, readonly protocols?: string | string[]) {
		FakeWebSocket.instances.push(this);
	}

	send(message: string): void {
		if (this.readyState !== FakeWebSocket.OPEN) throw new Error("socket is not open");
		this.sent.push(message);
	}

	open(): void {
		this.readyState = FakeWebSocket.OPEN;
		this.onopen?.(new Event("open"));
	}

	message(message: string): void {
		this.onmessage?.(new MessageEvent("message", { data: message }));
	}

	close(): void {
		this.readyState = FakeWebSocket.CLOSED;
		this.onclose?.({} as CloseEvent);
	}
}

class Client implements WSDOMClient {
	readonly received: string[] = [];

	constructor(readonly send: (message: string) => void, readonly label: string) {}

	handleIncomingMessage(message: string): void {
		this.received.push(message);
	}
}

async function websocketAndMiddlewareTest(): Promise<void> {
	FakeWebSocket.instances = [];
	(globalThis as unknown as { WebSocket: typeof WebSocket }).WebSocket = FakeWebSocket as unknown as typeof WebSocket;
	const calls: string[] = [];
	const wrappers: ProtocolWrapper<string, string>[] = [
		{
			async start(context) {
				equal(await context.interact("credential"), "accepted", "interaction result");
				calls.push("start-a");
			},
			async outbound(message, context) {
				calls.push("out-a");
				await context.sendOutbound(`a:${message}`);
			},
			async inbound(message, context) {
				calls.push("in-a");
				await context.sendInbound(`a:${message}`);
			},
		},
		{
			async outbound(message, context) {
				calls.push("out-b");
				await context.sendOutbound(`b:${message}`);
			},
			async inbound(message, context) {
				calls.push("in-b");
				await context.sendInbound(`b:${message}`);
			},
		},
	];
	const transport = new WSDOMTransport(Client, ["test-client"], {
		transport: { kind: "websocket", url: "ws://example.test/ws", protocols: "wsdom" },
		wrappers,
		interact: async () => "accepted",
	});
	equal(transport.client.label, "test-client", "constructor arguments reach the generated client");
	transport.client.send("queued");
	await tick();
	await transport.start();
	const socket = FakeWebSocket.instances[0];
	equal(socket.protocols, "wsdom", "WebSocket protocols are forwarded");
	socket.open();
	await tick();
	equal(socket.sent[0], "b:a:queued", "outbound wrappers run in declaration order");
	socket.message("server");
	await tick();
	equal(transport.client.received[0], "a:b:server", "inbound wrappers run in reverse order");
	equal(calls.join(","), "out-a,out-b,start-a,in-b,in-a", "wrapper lifecycle and direction order");
	socket.close();
	await new Promise<void>((resolve) => setTimeout(resolve, 275));
	assert(FakeWebSocket.instances.length === 2, "closed WebSockets reconnect with backoff");
	await transport.close();
	await new Promise<void>((resolve) => setTimeout(resolve, 275));
	equal(FakeWebSocket.instances.length, 2, "close cancels pending reconnects");
}

async function longPollTest(): Promise<void> {
	const requests: string[][] = [];
	let calls = 0;
	const transport = new WSDOMTransport(Client, ["poll-client"], {
		transport: {
			kind: "long-poll",
			url: "https://example.test/poll",
			pollIntervalMs: 1,
			fetch: async (_url, init) => {
				requests.push(JSON.parse(String(init?.body)) as string[]);
				calls++;
				return {
					ok: true,
					status: 200,
					json: async () => (calls === 1 ? ["first", "second"] : []),
				} as Response;
			},
		},
	});
	transport.client.send("before-start");
	await tick();
	await transport.start();
	await new Promise<void>((resolve) => setTimeout(resolve, 15));
	equal(JSON.stringify(requests[0]), JSON.stringify(["before-start"]), "long-poll posts a JSON string array");
	equal(transport.client.received.join(","), "first,second", "long-poll delivers ordered incoming messages");
	assert(requests.length >= 2, "long-poll schedules the next fast-batch request");
	await transport.close();
	const requestCountAfterClose = requests.length;
	await new Promise<void>((resolve) => setTimeout(resolve, 10));
	equal(requests.length, requestCountAfterClose, "close stops future long-poll requests");
}

async function missingInteractionTest(): Promise<void> {
	const errors: unknown[] = [];
	const transport = new WSDOMTransport(Client, ["interaction-client"], {
		transport: { kind: "websocket", url: "ws://example.test/ws" },
		wrappers: [{ start: async (context) => { await context.interact("credential"); } }],
		onError: (error) => errors.push(error),
	});
	let rejected = false;
	try {
		await transport.start();
	} catch (error) {
		rejected = true;
		assert(error instanceof Error && error.message.includes("requires host interaction"), "missing interaction error");
	}
	assert(rejected, "interactive wrapper startup fails without a host callback");
	equal(errors.length, 1, "interactive startup failure is reported");
	await transport.close();
}

async function main(): Promise<void> {
	await websocketAndMiddlewareTest();
	await longPollTest();
	await missingInteractionTest();
	console.log("transport tests passed");
}

void main();
