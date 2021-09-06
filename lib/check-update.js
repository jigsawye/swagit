const checkForUpdate = require('update-check');
const { bgRed } = require('chalk');

const pkg = require('../package.json');

module.exports = async () => {
  const update = await checkForUpdate(pkg);

  if (update) {
    console.log(
      `${bgRed('UPDATE AVAILABLE')} The latest version of \`release\` is ${
        update.latest
      }`
    );
  }
};
