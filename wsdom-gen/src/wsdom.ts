type Id = number;
type Value = unknown;
type SendMessage = (msg: string) => void;

export function WSDOMConnectWebSocket(wsUrl: string | URL, wsProtocols?: string | string[]): WSDOM {
	const ws = new WebSocket(wsUrl, wsProtocols);
    const q: string[] = [];
	const wsdom = new WSDOM((msg: string) => {
        if(ws.readyState !== WebSocket.OPEN){
            q.push(msg);
            return;
        }
		ws.send(msg);
	});
	ws.onopen = () => {
        for(const msg of q){
            ws.send(msg);
        }
        q.length = 0;
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
export class WSDOM{
	#sender: SendMessage;
	#values: Map<Id, { value: Value, error: boolean }>;
    #callbacks: Map<Id,(value: Value) => void>;
    #next_value: Id;
    public handleIncomingMessage(msg: string) {
		const fn = new Function('_w', msg);
		fn(this.#api);
	}
	constructor(sender: SendMessage) {
		this.#sender = sender;
		this.#values = new Map();
        this.#callbacks = new Map();
        this.#next_value = Number.MAX_SAFE_INTEGER;
        Object.freeze(this);
	}
    #allocate (v: Value): Id {
        var i = this.#next_value;
        this.#next_value--;
        this.#values.set(i,{value: v, error: false});
        return i;
    }
    #a = this.#allocate;
	#g (id: Id): Value {
		var w = this.#values.get(id);
		if (w?.error) {
			throw w.value
		} else {
			return w?.value
		}
	}
	#s (id: Id, value: Value) {
		this.#values.set(id, { value, error: false });
	}
	#d (id: Id) {
		this.#values.delete(id);
	}
	#r (id: Id, val: Value) {
		const valJson = JSON.stringify(val);
		(this.#sender)(`p${id}:${valJson}`);
	}
    #rp (id: Id, val: Value) {
        var cb = this.#callbacks.get(id);
        if(cb !== undefined){
            cb(val)
        }
	}
	#c (id: Id): {value: Value} | {slot: Id} | undefined  {
		var w = this.#values.get(id);
		if(w?.error){
			return {slot: this.#allocate(w.value)};
		}else{
			return {value: w?.value}
		}
	}
	#e (id: Id, value: Value) {
		this.#values.set(id, { value, error: true })
	}
    #x: {[key: string]: Value} = Object.freeze({__proto__: null, $$x});

    #api = Object.freeze({
        __proto__: null,
        a: this.#a.bind(this),
        g: this.#g.bind(this),
        s: this.#s.bind(this),
        d: this.#d.bind(this),
        r: this.#r.bind(this),
        rp: this.#rp.bind(this),
        c: this.#c.bind(this),
        e: this.#e.bind(this),
        x: this.#x,
    });

    static{
        Object.freeze(WSDOM.prototype);
        Object.freeze(WSDOM);
    }

    $$e;

    
}