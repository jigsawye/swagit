const git = require('simple-git/promise');

const getGitBranches = async () => {
  const result = await git().branch();
  return result;
};

exports.checkGit = async () => {
  try {
    const { current } = await git().status();
    return current;
  } catch (err) {
    return false;
  }
};

exports.getBranches = async () => {
  const { current, all } = await getGitBranches();
  return all.filter(branch => branch !== current);
};

exports.deleteBranches = async branches => {
  const result = await git().branch(['-D', ...branches]);
  return result;
};

exports.checkout = async branch => {
  const result = await git().checkout(branch);
  return result;
};
