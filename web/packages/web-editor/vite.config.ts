import fs from "node:fs";
import path from "node:path";
import { defineConfig, type Plugin } from "vite";

const rootDir = __dirname;
const outDir = path.resolve(rootDir, "dist");
const staticAssets = [
  {
    mountPath: "/examples/",
    sourceDir: path.resolve(rootDir, "../../../examples"),
    outputDir: path.resolve(outDir, "examples"),
  },
  {
    mountPath: "/std/",
    sourceDir: path.resolve(rootDir, "../../../std"),
    outputDir: path.resolve(outDir, "std"),
  },
];

const contentTypes = new Map<string, string>([
  [".json", "application/json; charset=utf-8"],
  [".pol", "text/plain; charset=utf-8"],
]);

const getContentType = (filepath: string): string => contentTypes.get(path.extname(filepath)) ?? "application/octet-stream";

const resolveStaticAsset = (url: string): string | undefined => {
  for (const asset of staticAssets) {
    if (!url.startsWith(asset.mountPath)) continue;

    const relativePath = decodeURIComponent(url.slice(asset.mountPath.length));
    const filepath = path.resolve(asset.sourceDir, relativePath);
    const relativeToRoot = path.relative(asset.sourceDir, filepath);

    if (relativeToRoot.startsWith("..") || path.isAbsolute(relativeToRoot)) {
      return undefined;
    }

    return filepath;
  }

  return undefined;
};

const staticAssetPlugin = (): Plugin => ({
  name: "polarity-static-assets",
  configureServer(server) {
    server.middlewares.use((req, res, next) => {
      const url = req.url?.split("?")[0];
      if (!url) {
        next();
        return;
      }

      if (url === "/editor") {
        res.statusCode = 302;
        res.setHeader("Location", "/editor/");
        res.end();
        return;
      }

      const filepath = resolveStaticAsset(url);
      if (!filepath) {
        next();
        return;
      }

      if (!fs.existsSync(filepath) || !fs.statSync(filepath).isFile()) {
        res.statusCode = 404;
        res.end();
        return;
      }

      res.setHeader("Content-Type", getContentType(filepath));
      fs.createReadStream(filepath).pipe(res);
    });
  },
  closeBundle() {
    for (const asset of staticAssets) {
      fs.rmSync(asset.outputDir, { force: true, recursive: true });
      fs.cpSync(asset.sourceDir, asset.outputDir, { recursive: true });
    }
  },
});

export default defineConfig({
  build: {
    outDir: "dist",
    rollupOptions: {
      input: path.resolve(rootDir, "editor/index.html"),
    },
    target: "es2022",
  },
  plugins: [staticAssetPlugin()],
  define: {
    DEBUG: "false",
  },
  resolve: {
    alias: {
      "polarity-lang-lsp-web": path.resolve(rootDir, "../lsp-web/src/lsp-web"),
      path: "path-browserify",
      vm: "vm-browserify",
      fs: path.resolve(rootDir, "src/shims/empty.ts"),
      module: path.resolve(rootDir, "src/shims/empty.ts"),
      child_process: path.resolve(rootDir, "src/shims/empty.ts"),
      net: path.resolve(rootDir, "src/shims/empty.ts"),
      crypto: path.resolve(rootDir, "src/shims/empty.ts"),
    },
  },
  server: {
    open: "/editor/",
    port: 9000,
  },
});
