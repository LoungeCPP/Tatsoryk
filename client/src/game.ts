/// <reference path="../typings/browser.d.ts" />

import {GameInput, MousePos, getRelativeMouseCords} from './input';
import {GameWSTransport} from './transport';
import {GameScreen} from './gamescreen';
import {GameSocket, MessageData, Entity} from './protocol';

class Game {
    static tickPeriod: number = 1.0 / 60;

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
    // Updates without full state
    //

    update = (): void => {
        this.updatePlayerStates();
        this.updateBulletStates();
        this.doBulletCollisionChecks();
    }

    updatePlayerStates(): void {
        var minDistance = this.welcomeMessage.size;
        var minPlayerDistanceSq = 4 * minDistance * minDistance;
        var maxDistanceX = this.canvas.width - this.welcomeMessage.size;
        var maxDistanceY = this.canvas.height - this.welcomeMessage.size;
        this.game.state.alivePlayers.forEach((player: Entity): void => {
            if (player.direction != null) {
                var move = player.direction.clone().multiplyScalar(this.welcomeMessage.speed);
                var newpos = player.position.clone().add(move);
                if (!this.game.state.alivePlayers.some((cmpPlayer: Entity) => {
                    return cmpPlayer.id != player.id && cmpPlayer.distanceSq(newpos) <= minPlayerDistanceSq;
                })) {
                    player.position.x = Math.min(Math.max(newpos.x, minDistance), maxDistanceX);
                    player.position.y = Math.min(Math.max(newpos.y, minDistance), maxDistanceY);
                }
            }
        });
    }

    updateBulletStates(): void {
        this.game.state.aliveBullets.forEach((bullet: Entity): void => {
            if (bullet.direction != null) {
                bullet.position.x += bullet.direction.x * this.welcomeMessage.bulletSpeed;
                bullet.position.y += bullet.direction.y * this.welcomeMessage.bulletSpeed;
            }
        });
    }

    doBulletCollisionChecks(): void {
        var killedPlayers: Array<number> = [];
        var destroyedBullets: Array<number> = [];
        this.game.state.aliveBullets.forEach((bullet: Entity): void => {
            if (this.bulletOutOfBounds(bullet)) {
                destroyedBullets.push(bullet.id);
                return; // if the bullet just went OOB, no point checking for player hits... right?
            }
            this.game.state.alivePlayers.forEach((player: Entity): void => {
                if (this.bulletHitPlayer(bullet, player)) {
                    destroyedBullets.push(bullet.id);
                    killedPlayers.push(player.id);
                }
            });
        });

        let removeKilledEntity = (killedEntities: Array<number>): ((entity: Entity) => boolean) => {
            return function(entity: Entity): boolean {
                return killedEntities.indexOf(entity.id) == -1;
            };
        };
        this.game.state.alivePlayers = this.game.state.alivePlayers.filter(removeKilledEntity(killedPlayers));
        this.game.state.aliveBullets = this.game.state.aliveBullets.filter(removeKilledEntity(destroyedBullets));
    }

    bulletOutOfBounds(bullet: Entity): boolean {
        var pos = bullet.position;
        return pos.x < 0 || pos.y < 0 || pos.x > this.canvas.width || pos.y > this.canvas.height;
    }

    bulletHitPlayer(bullet: Entity, player: Entity): boolean {
        var distanceSq = bullet.distanceSq(player);
        var collisionRange = this.welcomeMessage.size + this.welcomeMessage.bulletSize;
        // calculation is done on distance squared as it avoids the relatively costly square toot
        return distanceSq < collisionRange * collisionRange;
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
        alert('Server told us to go away because of: ' + msg.reason);
        this.disconnect();
    }

    handlePlayerJoined = (msg: MessageData.PlayerJoined): void => {
        ++this.game.state.playerCount;
    }

    handlePlayerLeft = (msg: MessageData.PlayerLeft): void => {
        --this.game.state.playerCount;
        this.game.state.alivePlayers = this.game.state.alivePlayers.filter((player: Entity): boolean => {
            return player.id != msg.id;
        });
    }

    handleShotsFired = (msg: MessageData.ShotsFired): void => {
        this.game.state.aliveBullets.push(new Entity(msg.bulletID, msg.position, msg.aim));
    }

    handlePlayerSpawned = (msg: MessageData.PlayerSpawned): void => {
        this.game.state.alivePlayers.push(new Entity(msg.id, msg.position));
    }

    handlePlayerDestroyed = (msg: MessageData.PlayerDestroyed): void => {
        this.game.state.alivePlayers = this.game.state.alivePlayers.filter((player: Entity): boolean => {
            return player.id != msg.id;
        });
        if (msg.bulletID != null) {
            this.game.state.aliveBullets = this.game.state.aliveBullets.filter((bullet: Entity): boolean => {
                return bullet.id != msg.bulletID;
            });
        }
    }

    handlePlayerMoving = (msg: MessageData.PlayerMoving): void => {
        var player = this.game.state.alivePlayers.find((player: Entity): boolean => {
            return player.id == msg.id;
        });
        player.position = msg.position;
        player.direction = msg.direction;
    }

    handlePlayerStopped = (msg: MessageData.PlayerStopped): void => {
        var player = this.game.state.alivePlayers.find((player: Entity): boolean => {
            return player.id == msg.id;
        });
        player.position = msg.position;
        player.direction = null;
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

    updateInterval: number;

    // Create the game given a welcome message and an initial state.
    // Welcome must be a welcome message.
    // initialState must be a world state message.
    setupGame(welcome: MessageData.Welcome, initialState: MessageData.WorldState): void {
        // TODO
        this.game = new GameScreen(welcome.id, welcome.size, welcome.bulletSize, initialState, this.socket);
        this.bindInput();
        this.updateInterval = setInterval(this.update, Game.tickPeriod * 1000);
    }

    stopGame = (): void => {
        // TODO
        clearInterval(this.updateInterval);
        this.updateInterval = null;
        this.unbindInput();
        this.game = null;
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
    }

    unbindInput(): void {
        if (this.keyboard !== null) {
            this.keyboard.reset();
            this.keyboard = null;
        }
        this.canvas.removeEventListener('mousemove', this.processMouseMove);
        this.canvas.removeEventListener('mousedown', this.processMouseDown);
    }
}

new Game();
