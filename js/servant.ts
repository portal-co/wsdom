
type Id = number;
type Value = unknown;
type SendMessage = (msg: string) => void;

export function WSDOMConnectWebSocket(wsUrl: string | URL, wsProtocols?: string | string[]): WSDOM {
	const ws = new WebSocket(wsUrl, wsProtocols);
	const wsdom = new WSDOM((msg: string) => {
		ws.send(msg);
	});
	ws.onopen = () => {
		console.debug("WSDOM WebSocket connection open!");
		console.debug("WebSocket object", ws);
		console.debug("WSDOM object", wsdom);
	}
	ws.onmessage = (msg: MessageEvent<string>) => {
		wsdom.handleIncomingMessage(msg.data);
	};
	ws.onclose = (ev: CloseEvent) => {
		console.debug("WSDOM WebSocket closed", ev);
	}
	ws.onerror = (ev: Event) => {
		console.warn("WSDOM WebSocket errored", ev);
	}
    return wsdom;
}
export class WSDOM {
	public internal: WSDOMCore;
	constructor(sendMessage: SendMessage) {
		this.internal = new WSDOMCore(sendMessage);
	}
	public handleIncomingMessage(msg: string) {
		const fn = new Function('_w', msg);
		fn(this.internal);
	}
    
}
export class WSDOMCore{
	public sender: SendMessage;
	private values: Map<Id, { value: Value, error: boolean }>;
    public callbacks: Map<Id,(value: Value) => void>;
    private next_value: Id;
	constructor(sender: SendMessage) {
		this.sender = sender;
		this.values = new Map();
        this.callbacks = new Map();
        this.next_value = Number.MAX_SAFE_INTEGER;
	}
    public allocate = (v: Value): Id => {
        var i = this.next_value;
        this.next_value--;
        this.values.set(i,{value: v, error: false});
        return i;
    }
	public g = (id: Id): Value => {
		var w = this.values.get(id);
		if (w?.error) {
			throw w.value
		} else {
			return w?.value
		}
	}
	public s = (id: Id, value: Value) => {
		this.values.set(id, { value, error: false });
	}
	public d = (id: Id) => {
		this.values.delete(id);
	}
	public r = (id: Id, val: Value) => {
		const valJson = JSON.stringify(val);
		(this.sender)(`p${id}:${valJson}`);
	}
    public rp = (id: Id, val: Value) => {
        var cb = this.callbacks.get(id);
        if(cb !== undefined){
            cb(val)
        }
	}
	public c = (id: Id, slot: Id): {value: Value} | {slot: Id} | undefined  => {
		var w = this.values.get(id);
		if(w?.error){
			this.values.set(slot,{value: w.value, error: false});
			return {slot};
		}else{
			return {value: w?.value}
		}
	}
	public e = (id: Id, value: Value) => {
		this.values.set(id, { value, error: true })
	}
    public x: {[key: string]: Value} = {};
}
