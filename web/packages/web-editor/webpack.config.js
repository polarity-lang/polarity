// @ts-check

const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const path = require("path");
const webpack = require("webpack");

/**
 * @param {Record<string, unknown>} env
 * @param {{ mode?: string }} argv
 */
module.exports = (env, argv) => {
  const prod = argv.mode === "production";
  console.log(prod ? "Production mode" : "Development mode");

  /** @type {import("webpack").Configuration & { devServer?: import("webpack-dev-server").Configuration } } */
  const config = {
    experiments: {
      asyncWebAssembly: true,
    },
    mode: "production",
    target: "web",
    entry: {
      editor: "./src/index.ts",
      "editor.worker": "monaco-editor-core/esm/vs/editor/editor.worker.js",
    },
    resolve: {
      extensions: [".ts", ".js", ".json", ".ttf"],
      alias: {
        "polarity-lang-lsp-web": path.resolve(__dirname, "../lsp-web/src/lsp-web"),
      },
      fallback: {
        vm: require.resolve("vm-browserify"),
        module: false,
        fs: false,
        child_process: false,
        net: false,
        crypto: false,
        path: require.resolve("path-browserify"),
      },
    },
    output: {
      globalObject: "self",
      filename: "[name].bundle.js",
      path: path.resolve(__dirname, "dist"),
    },
    module: {
      rules: [
        {
          test: /\.ts?$/,
          loader: "esbuild-loader",
          options: {
            loader: "ts",
            target: "es2022",
            minify: true,
          },
        },
        {
          test: /\.css$/,
          use: ["style-loader", "css-loader"],
        },
        {
          test: /\.(woff|woff2|eot|ttf|otf)$/i,
          type: "asset/resource",
        },
        {
          test: /\.html$/,
          loader: "html-loader",
        },
      ],
    },
    plugins: [
      new webpack.ProgressPlugin(),
      new CleanWebpackPlugin(),
      new CopyWebpackPlugin({
        patterns: [
          { from: "../../../examples", to: "examples" },
          { from: "../../../std", to: "std" },
        ],
      }),
      new webpack.DefinePlugin({
        DEBUG: !prod,
      }),
      new HtmlWebpackPlugin({
        template: "assets/editor.html",
        filename: "editor/index.html",
        chunks: ["editor", "editor.worker"],
        scriptLoading: "defer",
      }),
    ],
    optimization: {
      minimize: prod,
      runtimeChunk: "single",
    },
    performance: {
      hints: false,
    },
    devServer: {
      static: {
        directory: path.join(__dirname, "dist"),
      },
      port: 9000,
      client: {
        progress: true,
        reconnect: false,
      },
      devMiddleware: {
        writeToDisk: true,
      },
    },
    // Otherwise webpack fails the build with:
    // WARNING in ../../node_modules/vscode/vscode/src/vs/amdX.js 142:14-31
    ignoreWarnings: [/Critical dependency: the request of a dependency is an expression/],
  };
  return config;
};
