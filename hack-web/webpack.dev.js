const path = require('path');
const { merge } = require('webpack-merge');
const common = require('./webpack.common.js');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = merge(common, {
  mode: 'development',
  plugins: [
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "."),
      extraArgs: '--profiling -- --features console_error_panic_hook',
      forceMode: 'production'
    }),
  ]
});
