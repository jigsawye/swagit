#!/usr/bin/env node

const args = require('args');
const inquirer = require('inquirer');
const { yellow, magenta } = require('chalk');
const nodeVersion = require('node-version');

const { info, success, error } = require('../lib/message-prefix');
const {
  checkGit,
  getBranches,
  deleteBranches,
  checkout,
} = require('../lib/git');

if (nodeVersion.major < 6) {
  console.error(
    `${error} Now requires at least version 6 of Node. Please upgrade!`
  );
  process.exit(1);
}

args.option('d', 'Select branches which you want to delete');

const flags = args.parse(process.argv);

const startCheckout = async () => {
  const choices = await getBranches();

  const { branch } = await inquirer.prompt([
    {
      type: 'list',
      name: 'branch',
      message: 'Which branch do you want to checkout?',
      choices,
    },
  ]);

  checkout(branch);

  console.log(`${success} Checkout current branch to ${magenta(branch)}`);
};

const startDeleteBranches = async () => {
  const choices = await getBranches();

  const { branches } = await inquirer.prompt([
    {
      type: 'checkbox',
      name: 'branches',
      message: 'Which branches do you want to delete?',
      choices,
    },
  ]);

  if (branches.length === 0) {
    console.log('No branch selected, exit.');
    process.exit(1);
  }

  const messageSuffix =
    branches.length === 1
      ? 'this branch?'
      : `those ${yellow.bold(branches.length)} branches?`;

  const { confirm } = await inquirer.prompt([
    {
      type: 'confirm',
      name: 'confirm',
      message: `Are you want to ${yellow.bold('DELETE')} ${messageSuffix}
  ${branches.join(', ')}`,
    },
  ]);

  if (confirm) {
    deleteBranches(branches);

    console.log(
      `${success} ${magenta(branches.length)} branches has been deleted.`
    );
  }
};

const main = async () => {
  const currentBranch = await checkGit();

  if (!currentBranch) {
    console.error(
      `${error} Not a git repository (or any of the parent directories)`
    );
    process.exit(1);
  }

  console.log(`${info} Current branch is ${magenta(currentBranch)}`);

  if (flags.d) {
    await startDeleteBranches();
  } else {
    await startCheckout();
  }
};

main();
