import EventEmitter = require("wolfy87-eventemitter");
import {Message} from './protocol';

//
// Low-level transport code
//
// Transport handles connection management (with automatic reconnection on error), and
// the encoding of the received/sent frames. Outgoing frames can be sent with `send(object)` method, and
// incoming ones received through `message(object)` event.
//
// We currently use WebSocket-based transport and JSON encoding.
//

export class GameWSTransport extends EventEmitter {
    address: string;
    reconnInterval: number;

    socket: WebSocket = null;
    pending: number = null;
    explicitDisconnect: boolean = false;

    constructor(address: string, reconnInterval?: number) {
        super();
        this.address = address;
        this.reconnInterval = reconnInterval || 10000;
    }

    disconnect(): void {
        console.log('WSTransport: disconnect');
        this.explicitDisconnect = true;

        if (this.pending !== null) {
            clearTimeout(this.pending);
            this.pending = null;
        }

        if (this.socket !== null) {
            this.socket.close();
            this.socket = null;
        }
    }

    connect(): void {
        console.log('WSTransport: connect');
        this.explicitDisconnect = false;

        if (this.socket !== null) {
            console.error('WSTransport: connect() called when already connected');
            return;
        }

        if (this.pending !== null) {
            clearTimeout(this.pending);
            this.pending = null;
        }

        console.log('WSTransport: trying to connect to %s', this.address);
        this.socket = new WebSocket(this.address);

        this.socket.onopen = (): void => {
            console.log('WSTransport: connected to %s', this.address);
            super.emitEvent('connect');
        };

        this.socket.onmessage = (e) => {
            var message = JSON.parse(e.data);
            super.emitEvent('message', [message]);
        };

        this.socket.onclose = (): void => {
            this.socket = null;

            console.log('WSTransport: disconnected from %s', this.address);
            super.emitEvent('disconnect');

            if (this.pending === null && !this.explicitDisconnect) {
                console.log('WSTransport: trying to reconnect in %dms', this.reconnInterval);

                this.pending = setTimeout((): void => {
                    if (this.explicitDisconnect) return;
                    this.connect();
                }, this.reconnInterval);
            }
        };

        this.socket.onerror = (e) => {
            console.error('WSTransport: error %o', e);
            super.emitEvent('error', [e]);
        };
    }

    send(message: Message): void {
        if (this.socket === null) {
            console.error('WSTransport: send() called when not connected');
            return;
        }

        var frame = JSON.stringify(message);
        this.socket.send(frame);
    }
}
