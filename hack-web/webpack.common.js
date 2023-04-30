const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const webpack = require('webpack');

module.exports = {
  entry: './www/index.ts',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bundle.js',
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
      {
        test: /\.hack$/,
        type: 'asset/resource',
      }
    ]
  },
  plugins: [
    new HtmlWebpackPlugin({ template: './www/template.html', title: 'Hack Emulator' }),
    new webpack.ProvidePlugin({
      TextDecoder: ['text-encoding', 'TextDecoder'],
      TextEncoder: ['text-encoding', 'TextEncoder']
    })
  ],
  resolve: {
    extensions: ['.js', '.ts'],
  },
  experiments: {
    asyncWebAssembly: true
  }
};
