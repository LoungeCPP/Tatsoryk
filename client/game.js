(function() {
    'use strict';
    var Game = window.Game = window.Game || {};

    //
    // Evil global state
    //

    var transport = null;
    var socket = null;

    // TODO player-related state
    var canvas = null;
    var lastTime = null;
    var keyboard = null;

    //
    // Input handling
    //

    var INPUT_MOVE_UP    = 0;
    var INPUT_MOVE_DOWN  = 1;
    var INPUT_MOVE_LEFT  = 2;
    var INPUT_MOVE_RIGHT = 3;
    var INPUT_FIRE       = 4;

    var onKeyDown = function(input) {
        // TODO maintain a proper movement vector
        socket.startMoving(Game.makeVector(0, 1));
    };

    var onKeyUp = function(input) {
        // TODO maintain a proper movement vector
        socket.stopMoving();
    };

    var onMouseInput = function(input, x, y) {
        // TODO calculate aim vector
        socket.fire(Game.makeVector(1, 1));
    };

    var bindInput = function() {
        var bind = function(key, input) {
            keyboard.register_combo({
                keys:            key,
                on_keydown:      onKeyDown.bind(null, input),
                on_keyup:        onKeyUp.bind(null, input),
                prevent_default: true,
                prevent_repeat:  true,
            });
        };

        keyboard = new window.keypress.Listener();
        bind('w', INPUT_MOVE_UP);
        bind('s', INPUT_MOVE_DOWN);
        bind('a', INPUT_MOVE_LEFT);
        bind('d', INPUT_MOVE_RIGHT);
    };

    var unbindInput = function() {
        if (keyboard !== null) {
            keyboard.reset();
        }
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

    var update = function(timeDelta) {
        // TODO
    };

    //
    // Rendering
    //

    var draw = function(context, width, height, timeDelta) {
        // TODO drawing
    };

    var frame = function(time) {
        if (lastTime === null) {
            lastTime = time;
        }

        var delta = time - lastTime;
        lastTime = time;

        update(delta);

        var context = canvas.getContext('2d');
        context.save();
        context.clearRect(0, 0, canvas.width, canvas.height);
        draw(context, canvas.width, canvas.height, delta);
        context.restore();

        window.requestAnimationFrame(frame);
    };

    //
    // Networking
    //

    var handleWelcome = function(msg) {
        // TODO do stuff~
    };

    var handleGoAway = function(msg) {
        alert('Server terminated our connection: ' + msg.reason);
    };

    var handlePlayerJoined = function(msg) {
        // TODO do stuff~
    };

    var handlePlayerLeft = function(msg) {
        // TODO do stuff~
    };

    var handleShotsFired = function(msg) {
        // TODO do stuff~
    };

    var handlePlayerSpawned = function(msg) {
        // TODO do stuff~
    };

    var handlePlayerDestroyed = function(msg) {
        // TODO do stuff~
    };

    var handlePlayerMoving = function(msg) {
        // TODO do stuff~
    };

    var handlePlayerStopped = function(msg) {
        // TODO do stuff~
    };

    var handleWorldState = function(msg) {
        // TODO do stuff~
    };

    var connect = function(address) {
        transport = new Game.WSTransport(address);
        transport.addListener('connect',    setupGame);
        transport.addListener('disconnect', stopGame);

        socket = new Game.Socket(transport);
        socket.addListener('welcome',          handleWelcome);
        socket.addListener('go_away',          handleGoAway);
        socket.addListener('player_joined',    handlePlayerJoined);
        socket.addListener('player_left',      handlePlayerLeft);
        socket.addListener('shots_fired',      handleShotsFired);
        socket.addListener('player_spawned',   handlePlayerSpawned);
        socket.addListener('player_destroyed', handlePlayerDestroyed);
        socket.addListener('player_moving',    handlePlayerMoving);
        socket.addListener('player_stopped',   handlePlayerStopped);
        socket.addListener('world_state',      handleWorldState);

        transport.connect();
    };

    var disconnect = function() {
        if (transport !== null) {
            transport.disconnect();
        }

        transport = null;
        socket    = null;
    };

    //
    // DOM events
    //

    var onConnectClick = function(e) {
        var address = document.getElementById('server-address').value;

        disconnect();
        connect(address);
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

        // TODO maybe figure out good way to fill available viewport

        window.requestAnimationFrame(frame);
    };

    document.addEventListener('DOMContentLoaded', onDOMReady);
})();
