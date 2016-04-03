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

    var game = null;

    // Get the mouse coords relative to the canvas
    var getRelativeMouseCords = function(event) {
        var rect = canvas.getBoundingClientRect();
        var borderWidth = 1;
        return {
            x: event.clientX - rect.left - borderWidth,
            y: event.clientY - rect.top - borderWidth,
        }
    };

    var processMouseDown = function(event) {
        var coords = getRelativeMouseCords(event);
        game.onMouseClick(coords.x, coords.y, Game.INPUT_FIRE);
    };

    var processMouseMove = function(event) {
        var coords = getRelativeMouseCords(event);
        game.onMouseMove(coords.x, coords.y);
    };

    var bindInput = function() {
        var bind = function(key, input) {
            keyboard.register_combo({
                keys:            key,
                on_keydown:      game.onKeyDown.bind(game, input),
                on_keyup:        game.onKeyUp.bind(game, input),
                prevent_default: true,
                prevent_repeat:  true,
            });
        };

        keyboard = new window.keypress.Listener();
        bind('w', Game.INPUT_MOVE_UP);
        bind('s', Game.INPUT_MOVE_DOWN);
        bind('a', Game.INPUT_MOVE_LEFT);
        bind('d', Game.INPUT_MOVE_RIGHT);

        canvas.addEventListener('mousedown', processMouseDown);
        canvas.addEventListener('mousemove', processMouseMove);
    };

    var unbindInput = function() {
        if (keyboard !== null) {
            keyboard.reset();
        }
        keyboard = null;
        canvas.removeEventListener('mousemove', processMouseMove);
        canvas.removeEventListener('mousedown', processMouseDown);
    };

    //
    // Game logic
    //

    // Create the game given a welcome message and an initial state.
    // Welcome must be a welcome message.
    // initialState must be a world state message.
    var setupGame = function(welcome, initialState) {
        // TODO
        game = new Game.GameScreen(welcome.id, welcome.size, welcome.bulletSize, initialState, socket);
        bindInput();
    };

    var stopGame = function() {
        // TODO
        unbindInput();
    };

    //
    // Rendering
    //

    var frame = function(time) {
        if (lastTime === null) {
            lastTime = time;
        }

        var delta = time - lastTime;
        lastTime = time;

        var context = canvas.getContext('2d');
        context.save();
        context.clearRect(0, 0, canvas.width, canvas.height);

        if (game != null) {
            game.draw(context);
        }

        context.restore();

        window.requestAnimationFrame(frame);
    };

    //
    // Networking
    //

    var welcomeMessage = null;

    // One issue is that a welcome message is not sufficient to start a game.
    // We need a world state message as well.
    // So we just hold onto our welcome message if we get one at the start.
    // TODO: Add an initial state field into the welcome message.
    var handleWelcome = function(msg) {
        welcomeMessage = msg;
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
        // If the game isn't created yet, create it now.
        if (game == null) {
            console.assert(welcomeMessage != null, 'Must have gotten a welcome message before a world state message');
            setupGame(welcomeMessage, msg);
        } else {
            game.updateState(msg);
        }
    };

    var connect = function(address) {
        transport = new Game.WSTransport(address);
        transport.addListener('connect',    function(){}); // We don't do anything on connect.
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

        var address = document.getElementById('server-address');
        address.value = address.value.replace('localhost:8000', /http(?:s?):\/\/(.*)*\//.exec(document.URL)[1]);
    };

    document.addEventListener('DOMContentLoaded', onDOMReady);
})();
