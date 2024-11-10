#!/usr/bin/env node

const { platform, arch } = process;

function getBinaryPath() {
  const platform_arch = `${platform}-${arch}`;
  const binary_path = {
    'darwin-x64': '@swagit/darwin-x64',
    'darwin-arm64': '@swagit/darwin-arm64',
    'linux-x64': '@swagit/linux-x64',
    'linux-arm64': '@swagit/linux-arm64',
    'win32-x64': '@swagit/win32-x64',
  }[platform_arch];

  if (!binary_path) {
    throw new Error(`Unsupported platform: ${platform_arch}`);
  }

  return require.resolve(binary_path);
}

require('child_process')
  .spawn(getBinaryPath(), process.argv.slice(2), { stdio: 'inherit' })
  .on('exit', process.exit);
