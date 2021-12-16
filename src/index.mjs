import { WebSocketServer } from 'ws';
import Speaker from 'speaker';

const WS_PORT = 7619;
let wsId;
let wss = new WebSocketServer({ port: WS_PORT, host: "0.0.0.0" });

wss.on('connection', (ws, req) => {
    console.log("Got a connection...", req.headers["sec-websocket-key"]);
    if (wsId) {
        console.log(`Currently in use by ${wsId}. Aborting...`);
        ws.close();
        return;
    }
    wsId = req.headers["sec-websocket-key"];
    let speaker;
    let sampleRate;
    let channels;
    let bitDepth;
    ws.on('message', (message) => {
        if (!sampleRate) {
            sampleRate = parseInt(message.toString());
        }
        else if (!channels) {
            channels = parseInt(message.toString());
        }
        else if (!bitDepth) {
            bitDepth = parseInt(message.toString().match(/\d+/)[0]);
        } else {
            if (!speaker) {
                console.log(`Initializing a new speaker with:
sample rate: ${sampleRate}
bit depth: ${bitDepth}
channels: ${channels}
                `)
                speaker = new Speaker({ channels, bitDepth, sampleRate });
            }
            speaker.write(message);
        }
    });
    ws.on('close', _ => {
        if (speaker) speaker.close();
        console.log(wsId, "has disconnected");
        wsId = null;
    });
});
