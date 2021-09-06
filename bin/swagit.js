#!/usr/bin/env node

const args = require('args');
const fuzzy = require('fuzzy');
const inquirer = require('inquirer');
const { yellow, magenta } = require('chalk');

inquirer.registerPrompt(
  'autocomplete',
  require('inquirer-autocomplete-prompt')
);

const checkUpdate = require('../lib/check-update');
const checkNodeVersion = require('../lib/check-node-version');
const handleEsc = require('../lib/handle-esc');
const { info, success, error } = require('../lib/message-prefix');
const {
  checkGit,
  getBranches,
  deleteBranches,
  checkout,
} = require('../lib/git');

checkNodeVersion();

handleEsc();

args.option('d', 'Select branches which you want to delete');

const flags = args.parse(process.argv);

const search =
  (branches) =>
  (ans, input = '') =>
    new Promise((resolve) => {
      const fuzzyResult = fuzzy.filter(
        input,
        branches.map(({ value }) => value)
      );
      resolve(fuzzyResult.map((el) => el.original));
    });

const startCheckout = async (branches) => {
  const { branch } = await inquirer.prompt([
    {
      type: 'autocomplete',
      name: 'branch',
      message: 'Which branch do you want to checkout?',
      source: search(branches),
    },
  ]);

  checkout(branch);

  console.log(`${success} Checkout current branch to ${magenta(branch)}`);
};

const startDeleteBranches = async (branches) => {
  const { branches: selectedBranches } = await inquirer.prompt([
    {
      type: 'checkbox',
      name: 'branches',
      message: 'Which branches do you want to delete?',
      choices: branches,
    },
  ]);

  if (selectedBranches.length === 0) {
    console.log('No branch selected, exit.');
    process.exit(1);
  }

  const messageSuffix =
    selectedBranches.length === 1
      ? 'this branch?'
      : `those ${yellow.bold(selectedBranches.length)} branches?`;

  const { confirm } = await inquirer.prompt([
    {
      type: 'confirm',
      name: 'confirm',
      message: `Are you want to ${yellow.bold('DELETE')} ${messageSuffix}
  ${selectedBranches.join(', ')}`,
    },
  ]);

  if (confirm) {
    deleteBranches(selectedBranches);

    console.log(
      `${success} ${magenta(
        selectedBranches.length
      )} branches has been deleted.`
    );
  }
};

const checkGitRepository = async () => {
  const currentBranch = await checkGit();

  if (!currentBranch) {
    console.error(
      `${error} Not a git repository (or any of the parent directories)`
    );
    process.exit(1);
  }

  console.log(`${info} Current branch is ${magenta(currentBranch)}`);
};

const getGitBranches = async () => {
  const branches = await getBranches();

  if (branches.length === 0) {
    console.error(`${error} No other branches in the repository`);
    process.exit(1);
  }

  return branches;
};

const main = async () => {
  checkUpdate().catch(() => {});

  await checkGitRepository();

  const branches = await getGitBranches();

  if (flags.d) {
    await startDeleteBranches(branches);
  } else {
    await startCheckout(branches);
  }
};

main();
