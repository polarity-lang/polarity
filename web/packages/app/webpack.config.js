// @ts-check

const { CleanWebpackPlugin } = require("clean-webpack-plugin");
const CopyWebpackPlugin = require("copy-webpack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const path = require("path");
const webpack = require("webpack");

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
      app: "./src/index.ts",
      "editor.worker": "monaco-editor-core/esm/vs/editor/editor.worker.js",
    },
    resolve: {
      alias: {
        vscode: require.resolve("monaco-languageclient/vscode-compatibility"),
      },
      extensions: [".ts", ".js", ".json", ".ttf"],
      fallback: {
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
          test: /\.s[ac]ss$/i,
          use: ["style-loader", "css-loader", "sass-loader"],
        },
        {
          test: /\.(woff|woff2|eot|ttf|otf)$/i,
          type: "asset/resource",
        },
      ],
    },
    plugins: [
      new webpack.ProgressPlugin(),
      new CleanWebpackPlugin(),
      new CopyWebpackPlugin({
        patterns: [{ from: "../../../oopsla_examples", to: "examples" }],
      }),
      new HtmlWebpackPlugin({
        template: "assets/index.html",
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
  };
  return config;
};
