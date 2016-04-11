/// <reference path="../typings/browser.d.ts" />

import {GameInput, MousePos, getRelativeMouseCords} from './input';
import {GameWSTransport} from './transport';
import {GameScreen} from './gamescreen';
import {GameSocket, MessageData} from './protocol';

class Game {
    transport: GameWSTransport = null;
    socket: GameSocket = null;
    canvas: HTMLCanvasElement = null;
    lastTime: number = null;
    keyboard: Keypress.Listener = null;

    constructor() {
        document.addEventListener('DOMContentLoaded', this.onDOMReady);
    }

    //
    // DOM events
    //

    onDOMReady = (): void => {
        console.log('onDOMReady');
        this.canvas = <HTMLCanvasElement>document.getElementById('game-canvas');

        document.getElementById('connect').addEventListener('click', this.onConnectClick);
        document.getElementById('disconnect').addEventListener('click', this.onDisconnectClick);
        // TODO maybe figure out good way to fill available viewport

        window.requestAnimationFrame(this.frame);

        var address = <HTMLInputElement>document.getElementById('server-address');
        address.value = address.value.replace('localhost:8000', /http(?:s?):\/\/(.*)*\//.exec(document.URL)[1]);
    }

    onConnectClick = (e: Event): void => {
        var address = (<HTMLInputElement>document.getElementById('server-address')).value;

        this.disconnect();
        this.connect(address);
        e.stopPropagation();
    }

    onDisconnectClick = (e: Event): void => {
        this.disconnect();
        e.stopPropagation();
    }

    //
    // Networking
    //

    welcomeMessage: MessageData.Welcome;

    // One issue is that a welcome message is not sufficient to start a game.
    // We need a world state message as well.
    // So we just hold onto our welcome message if we get one at the start.
    // TODO: Add an initial state field into the welcome message.
    handleWelcome = (msg: MessageData.Welcome): void => {
        this.welcomeMessage = msg;
    }

    handleGoAway = (msg: MessageData.GoAway): void => {
        alert('Server terminated our connection: ' + msg.reason);
    }

    handlePlayerJoined = (msg: MessageData.PlayerJoined): void => {
        // TODO do stuff~
    }

    handlePlayerLeft = (msg: MessageData.PlayerLeft): void => {
        // TODO do stuff~
    }

    handleShotsFired = (msg: MessageData.ShotsFired): void => {
        // TODO do stuff~
    }

    handlePlayerSpawned = (msg: MessageData.PlayerSpawned): void => {
        // TODO do stuff~
    }

    handlePlayerDestroyed = (msg: MessageData.PlayerDestroyed): void => {
        // TODO do stuff~
    }

    handlePlayerMoving = (msg: MessageData.PlayerMoving): void => {
        // TODO do stuff~
    }

    handlePlayerStopped = (msg: MessageData.PlayerStopped): void => {
        // TODO do stuff~
    }

    handleWorldState = (msg: MessageData.WorldState): void => {
        // If the game isn't created yet, create it now.
        if (this.game == null) {
            console.assert(this.welcomeMessage != null, 'Must have gotten a welcome message before a world state message');
            this.setupGame(this.welcomeMessage, msg);
        } else {
            this.game.state = msg;
        }
    }

    connect(address: string): void {
        this.transport = new GameWSTransport(address);
        this.transport.addListener('connect', (): void => { });
        this.transport.addListener('disconnect', this.stopGame);

        this.socket = new GameSocket(this.transport);
        this.socket.addListener('welcome', this.handleWelcome);
        this.socket.addListener('go_away', this.handleGoAway);
        this.socket.addListener('player_joined', this.handlePlayerJoined);
        this.socket.addListener('player_left', this.handlePlayerLeft);
        this.socket.addListener('shots_fired', this.handleShotsFired);
        this.socket.addListener('player_spawned', this.handlePlayerSpawned);
        this.socket.addListener('player_destroyed', this.handlePlayerDestroyed);
        this.socket.addListener('player_moving', this.handlePlayerMoving);
        this.socket.addListener('player_stopped', this.handlePlayerStopped);
        this.socket.addListener('world_state', this.handleWorldState);

        this.transport.connect();
    }

    disconnect(): void {
        if (this.transport !== null) {
            this.transport.disconnect();
        }

        this.transport = null;
        this.socket = null;
    }

    //
    // Rendering
    //

    frame = (time: number): void => {
        if (this.lastTime === null) {
            this.lastTime = time;
        }

        var delta = time - this.lastTime;
        this.lastTime = time;

        var context = this.canvas.getContext('2d');
        context.save();
        context.clearRect(0, 0, this.canvas.width, this.canvas.height);

        if (this.game != null) {
            this.game.draw(context);
        }

        context.restore();

        window.requestAnimationFrame(this.frame);
    }

    //
    // Game logic
    //

    // Create the game given a welcome message and an initial state.
    // Welcome must be a welcome message.
    // initialState must be a world state message.
    setupGame(welcome: MessageData.Welcome, initialState: MessageData.WorldState): void {
        // TODO
        this.game = new GameScreen(welcome.id, welcome.size, welcome.bulletSize, initialState, this.socket);
        this.bindInput();
    }

    stopGame = (): void => {
        // TODO
        this.unbindInput();
    }

    //
    // Input handling

    game: GameScreen;

    processMouseDown = (event: MouseEvent): void => {
        var coords = getRelativeMouseCords(event, this.canvas);
        this.game.onMouseClick(coords.x, coords.y, GameInput.FIRE);
    }

    processMouseMove = (event: MouseEvent): void => {
        var coords = getRelativeMouseCords(event, this.canvas);
        this.game.onMouseMove(coords.x, coords.y);
    }

    bindInput(): void {
        var bind = (key: string, input: GameInput) => {
            this.keyboard.register_combo({
                keys: key,
                on_keydown: this.game.onKeyDown.bind(this.game, input),
                on_keyup: this.game.onKeyUp.bind(this.game, input),
                on_release: undefined,
                this: undefined,
                prevent_default: true,
                prevent_repeat: true,
                is_unordered: undefined,
                is_counting: undefined,
                is_exclusive: undefined,
                is_sequence: undefined,
                is_solitary: undefined,
            });
        };

        this.keyboard = new window.keypress.Listener();
        bind('w', GameInput.MOVE_UP);
        bind('s', GameInput.MOVE_DOWN);
        bind('a', GameInput.MOVE_LEFT);
        bind('d', GameInput.MOVE_RIGHT);

        this.canvas.addEventListener('mousedown', this.processMouseDown);
        this.canvas.addEventListener('mousemove', this.processMouseMove);
    };

    unbindInput(): void {
        if (this.keyboard !== null) {
            this.keyboard.reset();
            this.keyboard = null;
        }
        this.canvas.removeEventListener('mousemove', this.processMouseMove);
        this.canvas.removeEventListener('mousedown', this.processMouseDown);
    };
}

new Game();
