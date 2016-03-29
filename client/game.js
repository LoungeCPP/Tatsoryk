(function() {
    'use strict';

    //
    // Evil global state
    //

    // TODO player-related state

    var canvas = null;
    var socket = null;
    var address = null;
    var lastTime = null;
    var keyboard = null;

    var entities = null;

    //
    // Input handling
    //

    var INPUT_MOVE_UP    = 0;
    var INPUT_MOVE_DOWN  = 1;
    var INPUT_MOVE_LEFT  = 2;
    var INPUT_MOVE_RIGHT = 3;
    var INPUT_FIRE       = 4;

    var onKeyDown = function(event) {
        var message = {type: 'keydown', key: event.code};
        socket.send(JSON.stringify(message));
    };

    var onKeyUp = function(event) {
        var message = {type: 'keyup', key: event.code};
        socket.send(JSON.stringify(message));
    }

    var onMouseInput = function(input, x, y) {
    };

    var bindInput = function() {
        keyboard = new window.keypress.Listener();
    };

    var unbindInput = function() {
        keyboard.reset();
        keyboard = null;
    };

    //
    // Game logic
    //

    var setupGame = function() {
        // TODO
        bindInput();
    };

    var stopGame = function() {
        // TODO
        unbindInput();
    };

    //
    // Rendering
    //

    var draw = function(context, width, height) {
        if (entities != null) {
            for (var i = 0; i < entities.length; i++) {
                var entity = entities[i];

                context.fillStyle = 'black';
                context.fillRect(entity.x, entity.y, 10, 10);
            }
        }
    };

    var frame = function(time) {
        var context = canvas.getContext('2d');
        context.save();
        context.clearRect(0, 0, canvas.width, canvas.height);
        draw(context, canvas.width, canvas.height);
        context.restore();

        window.requestAnimationFrame(frame);
    };

    //
    // Networking
    //

    var handleMessage = function(msg) {
        // TODO do stuff~
        entities = msg.entities;
    };

    var sendMessage = function(msg) {
        msg = JSON.stringify(msg);
        socket.send(msg);
    };

    var connect = function() {
        console.log('Trying to connect to ' + address);

        socket = new WebSocket(address);

        socket.onopen = function() {
            console.log('Connected to ' + address);
            setupGame();
        };

        socket.onmessage = function(e) {
            var message = JSON.parse(e.data);
            handleMessage(message);
        };

        socket.onerror = socket.onclose = function() {
            console.log('Disconnected from ' + address);
            // TODO automatic reconnection
            stopGame();
        };
    };

    var disconnect = function() {
        if (socket !== null) {
            console.log('Disconnecting from ' + address);
            socket.close();
            socket = null;
        }
    };

    //
    // DOM events
    //

    var onConnectClick = function(e) {
        disconnect();

        address = document.getElementById('server-address').value;
        connect();

        e.stopPropagation();
    };

    var onDisconnectClick = function(e) {
        disconnect();
        e.stopPropagation();
    };

    var onDOMReady = function() {
        console.log('onDOMReady');
        canvas = document.getElementById('game-canvas');

        document.getElementById('connect').addEventListener('click', onConnectClick);
        document.getElementById('disconnect').addEventListener('click', onDisconnectClick);

        document.addEventListener('keydown', onKeyDown);
        document.addEventListener('keyup', onKeyUp);

        // TODO maybe figure out godo way to fill available viewport

        window.requestAnimationFrame(frame);
    };

    document.addEventListener('DOMContentLoaded', onDOMReady);
})();
