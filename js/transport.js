var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __classPrivateFieldSet = (this && this.__classPrivateFieldSet) || function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet = (this && this.__classPrivateFieldGet) || function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var _WSDOMTransport_instances, _WSDOMTransport_clientConstructor, _WSDOMTransport_args, _WSDOMTransport_options, _WSDOMTransport_wrappers, _WSDOMTransport_outbound, _WSDOMTransport_started, _WSDOMTransport_closed, _WSDOMTransport_wrappersStarted, _WSDOMTransport_webSocket, _WSDOMTransport_pollAbort, _WSDOMTransport_reconnectTimer, _WSDOMTransport_pollTimer, _WSDOMTransport_failures, _WSDOMTransport_sendFromClient, _WSDOMTransport_startWrappers, _WSDOMTransport_context, _WSDOMTransport_dispatchOutbound, _WSDOMTransport_dispatchInbound, _WSDOMTransport_connect, _WSDOMTransport_connectWebSocket, _WSDOMTransport_flushWebSocket, _WSDOMTransport_startPolling, _WSDOMTransport_poll, _WSDOMTransport_schedulePoll, _WSDOMTransport_handleFailure, _WSDOMTransport_scheduleReconnect, _WSDOMTransport_clearPollTimer, _WSDOMTransport_clearTimers, _WSDOMTransport_setStatus, _WSDOMTransport_reportError;
/**
 * Connects a sender-first generated WSDOM client to either a WebSocket or a
 * JSON-array long-poll endpoint. Transport selection is always explicit.
 */
export class WSDOMTransport {
    constructor(ClientConstructor, args, options) {
        var _a;
        _WSDOMTransport_instances.add(this);
        this.status = "idle";
        _WSDOMTransport_clientConstructor.set(this, void 0);
        _WSDOMTransport_args.set(this, void 0);
        _WSDOMTransport_options.set(this, void 0);
        _WSDOMTransport_wrappers.set(this, void 0);
        _WSDOMTransport_outbound.set(this, []);
        _WSDOMTransport_started.set(this, false);
        _WSDOMTransport_closed.set(this, false);
        _WSDOMTransport_wrappersStarted.set(this, false);
        _WSDOMTransport_webSocket.set(this, void 0);
        _WSDOMTransport_pollAbort.set(this, void 0);
        _WSDOMTransport_reconnectTimer.set(this, void 0);
        _WSDOMTransport_pollTimer.set(this, void 0);
        _WSDOMTransport_failures.set(this, 0);
        __classPrivateFieldSet(this, _WSDOMTransport_clientConstructor, ClientConstructor, "f");
        __classPrivateFieldSet(this, _WSDOMTransport_args, args, "f");
        __classPrivateFieldSet(this, _WSDOMTransport_options, options, "f");
        __classPrivateFieldSet(this, _WSDOMTransport_wrappers, (_a = options.wrappers) !== null && _a !== void 0 ? _a : [], "f");
        this.client = new (__classPrivateFieldGet(this, _WSDOMTransport_clientConstructor, "f"))((message) => __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_sendFromClient).call(this, message), ...__classPrivateFieldGet(this, _WSDOMTransport_args, "f"));
    }
    /** Starts wrappers and the explicitly configured physical transport. */
    start() {
        return __awaiter(this, void 0, void 0, function* () {
            if (__classPrivateFieldGet(this, _WSDOMTransport_closed, "f"))
                throw new Error("A closed WSDOMTransport cannot be restarted");
            if (__classPrivateFieldGet(this, _WSDOMTransport_started, "f"))
                return;
            __classPrivateFieldSet(this, _WSDOMTransport_started, true, "f");
            try {
                yield __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_startWrappers).call(this);
                __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_connect).call(this);
            }
            catch (error) {
                __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_reportError).call(this, error);
                __classPrivateFieldSet(this, _WSDOMTransport_started, false, "f");
                throw error;
            }
        });
    }
    /** Stops all activity, rejects future reconnects, and shuts wrappers down. */
    close() {
        var _a, _b, _c;
        return __awaiter(this, void 0, void 0, function* () {
            if (__classPrivateFieldGet(this, _WSDOMTransport_closed, "f"))
                return;
            __classPrivateFieldSet(this, _WSDOMTransport_closed, true, "f");
            __classPrivateFieldSet(this, _WSDOMTransport_started, false, "f");
            __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_clearTimers).call(this);
            (_a = __classPrivateFieldGet(this, _WSDOMTransport_pollAbort, "f")) === null || _a === void 0 ? void 0 : _a.abort();
            __classPrivateFieldSet(this, _WSDOMTransport_pollAbort, undefined, "f");
            const socket = __classPrivateFieldGet(this, _WSDOMTransport_webSocket, "f");
            __classPrivateFieldSet(this, _WSDOMTransport_webSocket, undefined, "f");
            if (socket && socket.readyState !== WebSocket.CLOSED)
                socket.close();
            __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_setStatus).call(this, "closed");
            if (!__classPrivateFieldGet(this, _WSDOMTransport_wrappersStarted, "f"))
                return;
            __classPrivateFieldSet(this, _WSDOMTransport_wrappersStarted, false, "f");
            for (let index = __classPrivateFieldGet(this, _WSDOMTransport_wrappers, "f").length - 1; index >= 0; index--) {
                try {
                    yield ((_c = (_b = __classPrivateFieldGet(this, _WSDOMTransport_wrappers, "f")[index]).stop) === null || _c === void 0 ? void 0 : _c.call(_b, __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_context).call(this, index)));
                }
                catch (error) {
                    __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_reportError).call(this, error);
                }
            }
        });
    }
}
_WSDOMTransport_clientConstructor = new WeakMap(), _WSDOMTransport_args = new WeakMap(), _WSDOMTransport_options = new WeakMap(), _WSDOMTransport_wrappers = new WeakMap(), _WSDOMTransport_outbound = new WeakMap(), _WSDOMTransport_started = new WeakMap(), _WSDOMTransport_closed = new WeakMap(), _WSDOMTransport_wrappersStarted = new WeakMap(), _WSDOMTransport_webSocket = new WeakMap(), _WSDOMTransport_pollAbort = new WeakMap(), _WSDOMTransport_reconnectTimer = new WeakMap(), _WSDOMTransport_pollTimer = new WeakMap(), _WSDOMTransport_failures = new WeakMap(), _WSDOMTransport_instances = new WeakSet(), _WSDOMTransport_sendFromClient = function _WSDOMTransport_sendFromClient(message) {
    void __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_dispatchOutbound).call(this, 0, message).catch((error) => __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_reportError).call(this, error));
}, _WSDOMTransport_startWrappers = function _WSDOMTransport_startWrappers() {
    var _a, _b;
    return __awaiter(this, void 0, void 0, function* () {
        for (let index = 0; index < __classPrivateFieldGet(this, _WSDOMTransport_wrappers, "f").length; index++) {
            yield ((_b = (_a = __classPrivateFieldGet(this, _WSDOMTransport_wrappers, "f")[index]).start) === null || _b === void 0 ? void 0 : _b.call(_a, __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_context).call(this, index)));
        }
        __classPrivateFieldSet(this, _WSDOMTransport_wrappersStarted, true, "f");
    });
}, _WSDOMTransport_context = function _WSDOMTransport_context(index) {
    return {
        sendOutbound: (message) => __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_dispatchOutbound).call(this, index + 1, message),
        sendInbound: (message) => __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_dispatchInbound).call(this, index - 1, message),
        interact: (request) => __awaiter(this, void 0, void 0, function* () {
            if (!__classPrivateFieldGet(this, _WSDOMTransport_options, "f").interact)
                throw new Error("This protocol wrapper requires host interaction");
            return __classPrivateFieldGet(this, _WSDOMTransport_options, "f").interact(request);
        }),
    };
}, _WSDOMTransport_dispatchOutbound = function _WSDOMTransport_dispatchOutbound(index, message) {
    return __awaiter(this, void 0, void 0, function* () {
        if (index >= __classPrivateFieldGet(this, _WSDOMTransport_wrappers, "f").length) {
            __classPrivateFieldGet(this, _WSDOMTransport_outbound, "f").push(message);
            __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_flushWebSocket).call(this);
            return;
        }
        const wrapper = __classPrivateFieldGet(this, _WSDOMTransport_wrappers, "f")[index];
        if (wrapper.outbound)
            yield wrapper.outbound(message, __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_context).call(this, index));
        else
            yield __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_dispatchOutbound).call(this, index + 1, message);
    });
}, _WSDOMTransport_dispatchInbound = function _WSDOMTransport_dispatchInbound(index, message) {
    return __awaiter(this, void 0, void 0, function* () {
        if (index < 0) {
            yield this.client.handleIncomingMessage(message);
            return;
        }
        const wrapper = __classPrivateFieldGet(this, _WSDOMTransport_wrappers, "f")[index];
        if (wrapper.inbound)
            yield wrapper.inbound(message, __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_context).call(this, index));
        else
            yield __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_dispatchInbound).call(this, index - 1, message);
    });
}, _WSDOMTransport_connect = function _WSDOMTransport_connect() {
    if (__classPrivateFieldGet(this, _WSDOMTransport_closed, "f") || !__classPrivateFieldGet(this, _WSDOMTransport_started, "f"))
        return;
    __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_setStatus).call(this, __classPrivateFieldGet(this, _WSDOMTransport_failures, "f") === 0 ? "connecting" : "reconnecting");
    if (__classPrivateFieldGet(this, _WSDOMTransport_options, "f").transport.kind === "websocket")
        __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_connectWebSocket).call(this);
    else
        __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_startPolling).call(this);
}, _WSDOMTransport_connectWebSocket = function _WSDOMTransport_connectWebSocket() {
    const { url, protocols } = __classPrivateFieldGet(this, _WSDOMTransport_options, "f").transport;
    let socket;
    try {
        socket = new WebSocket(url, protocols);
    }
    catch (error) {
        __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_handleFailure).call(this, error);
        return;
    }
    __classPrivateFieldSet(this, _WSDOMTransport_webSocket, socket, "f");
    socket.onopen = () => {
        if (socket !== __classPrivateFieldGet(this, _WSDOMTransport_webSocket, "f") || __classPrivateFieldGet(this, _WSDOMTransport_closed, "f"))
            return;
        __classPrivateFieldSet(this, _WSDOMTransport_failures, 0, "f");
        __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_setStatus).call(this, "open");
        __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_flushWebSocket).call(this);
    };
    socket.onmessage = (event) => {
        if (socket !== __classPrivateFieldGet(this, _WSDOMTransport_webSocket, "f") || typeof event.data !== "string") {
            if (typeof event.data !== "string")
                __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_reportError).call(this, new TypeError("WSDOM WebSocket frames must be strings"));
            return;
        }
        void __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_dispatchInbound).call(this, __classPrivateFieldGet(this, _WSDOMTransport_wrappers, "f").length - 1, event.data).catch((error) => __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_reportError).call(this, error));
    };
    socket.onerror = () => {
        if (socket === __classPrivateFieldGet(this, _WSDOMTransport_webSocket, "f"))
            __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_handleFailure).call(this, new Error("WSDOM WebSocket errored"));
    };
    socket.onclose = () => {
        if (socket !== __classPrivateFieldGet(this, _WSDOMTransport_webSocket, "f"))
            return;
        __classPrivateFieldSet(this, _WSDOMTransport_webSocket, undefined, "f");
        if (!__classPrivateFieldGet(this, _WSDOMTransport_closed, "f"))
            __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_scheduleReconnect).call(this);
    };
}, _WSDOMTransport_flushWebSocket = function _WSDOMTransport_flushWebSocket() {
    const socket = __classPrivateFieldGet(this, _WSDOMTransport_webSocket, "f");
    if (!socket || socket.readyState !== WebSocket.OPEN)
        return;
    while (__classPrivateFieldGet(this, _WSDOMTransport_outbound, "f").length > 0) {
        const message = __classPrivateFieldGet(this, _WSDOMTransport_outbound, "f")[0];
        try {
            socket.send(message);
            __classPrivateFieldGet(this, _WSDOMTransport_outbound, "f").shift();
        }
        catch (error) {
            __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_handleFailure).call(this, error);
            return;
        }
    }
}, _WSDOMTransport_startPolling = function _WSDOMTransport_startPolling() {
    __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_setStatus).call(this, "open");
    void __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_poll).call(this);
}, _WSDOMTransport_poll = function _WSDOMTransport_poll() {
    var _a, _b;
    return __awaiter(this, void 0, void 0, function* () {
        if (__classPrivateFieldGet(this, _WSDOMTransport_closed, "f") || !__classPrivateFieldGet(this, _WSDOMTransport_started, "f") || __classPrivateFieldGet(this, _WSDOMTransport_options, "f").transport.kind !== "long-poll")
            return;
        const messages = __classPrivateFieldGet(this, _WSDOMTransport_outbound, "f").splice(0);
        const controller = new AbortController();
        __classPrivateFieldSet(this, _WSDOMTransport_pollAbort, controller, "f");
        try {
            const transport = __classPrivateFieldGet(this, _WSDOMTransport_options, "f").transport;
            const headers = new Headers((_a = transport.requestInit) === null || _a === void 0 ? void 0 : _a.headers);
            if (!headers.has("content-type"))
                headers.set("content-type", "application/json");
            const response = yield ((_b = transport.fetch) !== null && _b !== void 0 ? _b : globalThis.fetch)(transport.url, Object.assign(Object.assign({}, transport.requestInit), { headers, method: "POST", body: JSON.stringify(messages), signal: controller.signal }));
            if (!response.ok)
                throw new Error(`WSDOM long-poll failed with HTTP ${response.status}`);
            const incoming = yield response.json();
            if (!Array.isArray(incoming) || !incoming.every((message) => typeof message === "string")) {
                throw new TypeError("WSDOM long-poll responses must be JSON string arrays");
            }
            __classPrivateFieldSet(this, _WSDOMTransport_failures, 0, "f");
            for (const message of incoming)
                yield __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_dispatchInbound).call(this, __classPrivateFieldGet(this, _WSDOMTransport_wrappers, "f").length - 1, message);
            __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_schedulePoll).call(this);
        }
        catch (error) {
            if (!controller.signal.aborted && !__classPrivateFieldGet(this, _WSDOMTransport_closed, "f")) {
                __classPrivateFieldGet(this, _WSDOMTransport_outbound, "f").unshift(...messages);
                __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_handleFailure).call(this, error);
            }
        }
        finally {
            if (__classPrivateFieldGet(this, _WSDOMTransport_pollAbort, "f") === controller)
                __classPrivateFieldSet(this, _WSDOMTransport_pollAbort, undefined, "f");
        }
    });
}, _WSDOMTransport_schedulePoll = function _WSDOMTransport_schedulePoll() {
    var _a;
    if (__classPrivateFieldGet(this, _WSDOMTransport_closed, "f") || !__classPrivateFieldGet(this, _WSDOMTransport_started, "f") || __classPrivateFieldGet(this, _WSDOMTransport_options, "f").transport.kind !== "long-poll")
        return;
    __classPrivateFieldSet(this, _WSDOMTransport_pollTimer, setTimeout(() => void __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_poll).call(this), (_a = __classPrivateFieldGet(this, _WSDOMTransport_options, "f").transport.pollIntervalMs) !== null && _a !== void 0 ? _a : 1000), "f");
}, _WSDOMTransport_handleFailure = function _WSDOMTransport_handleFailure(error) {
    __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_reportError).call(this, error);
    __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_scheduleReconnect).call(this);
}, _WSDOMTransport_scheduleReconnect = function _WSDOMTransport_scheduleReconnect() {
    var _a;
    var _b;
    if (__classPrivateFieldGet(this, _WSDOMTransport_closed, "f") || !__classPrivateFieldGet(this, _WSDOMTransport_started, "f") || __classPrivateFieldGet(this, _WSDOMTransport_reconnectTimer, "f"))
        return;
    __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_clearPollTimer).call(this);
    (_a = __classPrivateFieldGet(this, _WSDOMTransport_pollAbort, "f")) === null || _a === void 0 ? void 0 : _a.abort();
    __classPrivateFieldSet(this, _WSDOMTransport_pollAbort, undefined, "f");
    const socket = __classPrivateFieldGet(this, _WSDOMTransport_webSocket, "f");
    __classPrivateFieldSet(this, _WSDOMTransport_webSocket, undefined, "f");
    if (socket && socket.readyState !== WebSocket.CLOSED)
        socket.close();
    __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_setStatus).call(this, "reconnecting");
    const delay = Math.min(250 * 2 ** __classPrivateFieldGet(this, _WSDOMTransport_failures, "f"), 30000);
    __classPrivateFieldSet(this, _WSDOMTransport_failures, (_b = __classPrivateFieldGet(this, _WSDOMTransport_failures, "f"), _b++, _b), "f");
    __classPrivateFieldSet(this, _WSDOMTransport_reconnectTimer, setTimeout(() => {
        __classPrivateFieldSet(this, _WSDOMTransport_reconnectTimer, undefined, "f");
        __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_connect).call(this);
    }, delay), "f");
}, _WSDOMTransport_clearPollTimer = function _WSDOMTransport_clearPollTimer() {
    if (__classPrivateFieldGet(this, _WSDOMTransport_pollTimer, "f"))
        clearTimeout(__classPrivateFieldGet(this, _WSDOMTransport_pollTimer, "f"));
    __classPrivateFieldSet(this, _WSDOMTransport_pollTimer, undefined, "f");
}, _WSDOMTransport_clearTimers = function _WSDOMTransport_clearTimers() {
    if (__classPrivateFieldGet(this, _WSDOMTransport_reconnectTimer, "f"))
        clearTimeout(__classPrivateFieldGet(this, _WSDOMTransport_reconnectTimer, "f"));
    __classPrivateFieldSet(this, _WSDOMTransport_reconnectTimer, undefined, "f");
    __classPrivateFieldGet(this, _WSDOMTransport_instances, "m", _WSDOMTransport_clearPollTimer).call(this);
}, _WSDOMTransport_setStatus = function _WSDOMTransport_setStatus(status) {
    var _a, _b;
    if (this.status === status)
        return;
    this.status = status;
    (_b = (_a = __classPrivateFieldGet(this, _WSDOMTransport_options, "f")).onStatusChange) === null || _b === void 0 ? void 0 : _b.call(_a, status);
}, _WSDOMTransport_reportError = function _WSDOMTransport_reportError(error) {
    var _a, _b;
    (_b = (_a = __classPrivateFieldGet(this, _WSDOMTransport_options, "f")).onError) === null || _b === void 0 ? void 0 : _b.call(_a, error);
};
