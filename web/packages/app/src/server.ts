import init, { InitOutput, serve, ServerConfig } from "../assets/wasm/lsp_browser";
import { FromServer, IntoServer } from "./codec";

let server: null | Server;

export default class Server {
  readonly initOutput: InitOutput;
  readonly #intoServer: IntoServer;
  readonly #fromServer: FromServer;

  private constructor(initOutput: InitOutput, intoServer: IntoServer, fromServer: FromServer) {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    this.initOutput = initOutput;
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    this.#intoServer = intoServer;
    // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
    this.#fromServer = fromServer;
  }

  static async initialize(intoServer: IntoServer, fromServer: FromServer): Promise<Server> {
    if (null == server) {
      // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-assignment
      const initOutput = await init();
      server = new Server(initOutput, intoServer, fromServer);
    } else {
      console.warn("Server already initialized; ignoring");
    }
    return server;
  }

  async start(): Promise<void> {
    // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-assignment
    const config = new ServerConfig(this.#intoServer, this.#fromServer);
    // eslint-disable-next-line @typescript-eslint/no-unsafe-call
    await serve(config);
  }
}
