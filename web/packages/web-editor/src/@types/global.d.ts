declare const DEBUG: boolean;

declare module "vscode/assets" {
  export function registerAssets(assets: Record<string, string>): void;
}

declare module "*.wasm?url" {
  const url: string;
  export default url;
}
