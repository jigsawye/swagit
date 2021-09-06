const git = require('simple-git/promise');
const { white } = require('chalk');

exports.checkGit = async () => {
  try {
    const { current } = await git().status();
    return current;
  } catch (err) {
    return false;
  }
};

exports.getBranches = async () => {
  const { branches } = await git().branch();

  return Object.keys(branches)
    .filter((name) => !(branches[name].current || name.startsWith('remotes')))
    .map((name) => {
      const { commit, label } = branches[name];
      const description = white(`[${commit}] ${label}`);
      return { name: `${name} ${description}`, value: name };
    });
};

exports.deleteBranches = async (branches) => {
  const result = await git().branch(['-D', ...branches]);
  return result;
};

exports.checkout = async (branch) => {
  const result = await git().checkout(branch);
  return result;
};
