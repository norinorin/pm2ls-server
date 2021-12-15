import { WebSocketServer } from 'ws';
import opus from '@discordjs/opus';
import Speaker from 'speaker';

const WS_PORT = 7619;
let encoder = new opus.OpusEncoder(48000, 1);
let speaker = new Speaker({ channels: 1, bitDepth: 16, sampleRate: 48000 });
let wss = new WebSocketServer({ port: WS_PORT, host: "0.0.0.0" });

wss.on('connection', (ws, req) => {
    console.log("Got a connection...", req.headers["sec-websocket-key"]);
    ws.on('message', (message) => {
        const decoded = encoder.decode(message);
        speaker.write(decoded);
    });
    ws.on('close', _ => (
        console.log(req.headers["sec-websocket-key"], "has disconnected")
    ));
});