(function() {
    'use strict';
    var Game = window.Game = window.Game || {};

    //
    // Low-level transport code
    //
    // Transport handles connection management (with automatic reconnection on error), and
    // the encoding of the received/sent frames. Outgoing frames can be sent with `send(object)` method, and
    // incoming ones received through `message(object)` event.
    //
    // We currently use WebSocket-based transport, and JSON encoding.
    //

    Game.WSTransport = function(address, reconnInterval) {
        var self = this;
        var socket = null;
        var pending = null;
        var explicitDisconnect = false;

        reconnInterval = reconnInterval || 10000;

        self.disconnect = function() {
            console.log('WSTransport: disconnect');
            explicitDisconnect = true;

            if (pending !== null) {
                clearTimeout(pending);
                pending = null;
            }

            if (socket !== null) {
                socket.close();
                socket = null;
            }
        };

        self.connect = function() {
            console.log('WSTransport: connect');
            explicitDisconnect = false;

            if (socket !== null) {
                console.error('WSTransport: connect() called when already connected');
                return;
            }

            if (pending !== null) {
                clearTimeout(pending);
                pending = null;
            }

            console.log('WSTransport: trying to connect to %s', address);
            socket = new WebSocket(address);

            socket.onopen = function() {
                console.log('WSTransport: connected to %s', address);
                self.emitEvent('connect');
            };

            socket.onmessage = function(e) {
                var message = JSON.parse(e.data);
                self.emitEvent('message', [message]);
            };

            socket.onclose = function() {
                socket = null;

                console.log('WSTransport: disconnected from %s', address);
                self.emitEvent('disconnect');

                if (pending === null && !explicitDisconnect) {
                    console.log('WSTransport: trying to reconnect in %dms', reconnInterval);

                    pending = setTimeout(function() {
                        if (explicitDisconnect) return;
                        self.connect();
                    }, reconnInterval);
                }
            };

            socket.onerror = function(e) {
                console.error('WSTransport: error %o', e);
                self.emitEvent('error', [e]);
            };
        };

        self.send = function(message) {
            if (socket === null) {
                console.error('WSTransport: send() called when not connected');
                return;
            }

            var frame = JSON.stringify(message);
            socket.send(frame);
        };
    };

    heir.inherit(Game.WSTransport, EventEmitter);
})();
