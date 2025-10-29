import * as proto from "vscode-languageserver-protocol";

export default class Tracer {
  static client(message: string): void {
    DEBUG && console.log(`client -> server: ${message}`);
  }

  static server(input: string | proto.Message): void {
    const message: string = typeof input === "string" ? input : JSON.stringify(input);
    DEBUG && console.log(`server -> client: ${message}`);
  }
}
