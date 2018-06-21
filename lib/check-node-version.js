const { error } = require('./message-prefix');

module.exports = () => {
  const nodeVersion = require('node-version');

  if (nodeVersion.major < 6) {
    console.error(
      `${error} Now requires at least version 6 of Node. Please upgrade!`
    );
    process.exit(1);
  }
};
