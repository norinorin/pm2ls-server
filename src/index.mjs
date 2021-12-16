import { WebSocketServer } from 'ws';
import opus from '@discordjs/opus';
import Speaker from 'speaker';

const WS_PORT = 7619;
let inUse = false;
let wss = new WebSocketServer({ port: WS_PORT, host: "0.0.0.0" });

wss.on('connection', (ws, req) => {
    console.log("Got a connection...", req.headers["sec-websocket-key"]);
    if (inUse) {
        console.log("Another connection already exists! Aborting...");
        return;
    }
    inUse = true;
    let speaker;
    let encoder;
    ws.on('message', (message) => {
        // first message is the sample rate information
        if (!speaker) /* && !encoder */ {
            let sampleRate = parseInt(message.toString());
            console.log("Sample rate: ", sampleRate);
            speaker = new Speaker({ channels: 1, bitDepth: 16, sampleRate: sampleRate });
            encoder = new opus.OpusEncoder(sampleRate, 1);
            return;
        }

        speaker.write(encoder.decode(message));
    });
    ws.on('close', _ => {
        console.log(req.headers["sec-websocket-key"], "has disconnected");
        inUse = false;
    });
});
