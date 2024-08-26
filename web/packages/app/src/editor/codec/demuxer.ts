import * as vsrpc from "vscode-jsonrpc";

import Bytes from "./bytes";
import PromiseMap from "./map";
import Queue from "./queue";
import Tracer from "../tracer";

export default class StreamDemuxer extends Queue<Uint8Array> {
  readonly responses: PromiseMap<number | string, vsrpc.ResponseMessage> = new PromiseMap();
  readonly notifications: Queue<vsrpc.NotificationMessage> = new Queue<vsrpc.NotificationMessage>();
  readonly requests: Queue<vsrpc.RequestMessage> = new Queue<vsrpc.RequestMessage>();

  readonly #start: Promise<void>;

  constructor() {
    super();
    this.#start = this.start();
  }

  private async start(): Promise<void> {
    let contentLength: number | null = null;
    let buffer = new Uint8Array();

    while (true) {
      if (contentLength === null || buffer.length < contentLength) {
        const bytes = await this.next();
        buffer = Bytes.append(Uint8Array, buffer, bytes.value);
      }

      if (contentLength === null) {
        ({ contentLength, buffer } = this.parseContentLength(buffer));
      }

      if (contentLength === null) {
        continue;
      }

      if (buffer.length < contentLength) {
        continue;
      }

      const delimited = Bytes.decode(buffer.slice(0, contentLength));

      buffer = buffer.slice(contentLength);
      ({ contentLength, buffer } = this.parseContentLength(buffer));

      try {
        const message = JSON.parse(delimited) as vsrpc.Message;
        Tracer.server(message);
        this.demuxMessage(message);
      } catch (error) {
        console.error("Failed to parse message", error);
      }
    }
  }

  private parseContentLength(buffer: Uint8Array): { buffer: Uint8Array, contentLength: number | null } {
    const match = Bytes.decode(buffer).match(/^Content-Length:\s*(\d+)\s*/);
    if (match === null) return { buffer, contentLength: null };;

    const length = parseInt(match[1], 10);
    if (isNaN(length)) throw new Error("Invalid content length");

    buffer = buffer.slice(match[0].length);
    return { buffer, contentLength: length };
  }

  private demuxMessage(message: vsrpc.Message): void {
    if (vsrpc.Message.isResponse(message) && message.id !== null) {
      this.responses.set(message.id, message);
    } else if (vsrpc.Message.isNotification(message)) {
      this.notifications.enqueue(message);
    } else if (vsrpc.Message.isRequest(message)) {
      this.requests.enqueue(message);
    }
  }
}
